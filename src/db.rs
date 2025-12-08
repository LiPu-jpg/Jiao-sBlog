use sqlx::{PgPool, Error, Row};
use crate::models::*;
use chrono::Datelike;
use std::collections::HashMap;

/// -----------------------------
/// 合并文章 + 标签
/// -----------------------------
fn merge_articles(rows: Vec<sqlx::postgres::PgRow>) -> Vec<Article> {
    let mut map: HashMap<i32, Article> = HashMap::new();

    for row in rows {
        let id: i32 = row.get("id");

        let entry = map.entry(id).or_insert(Article {
            id,
            title: row.get("title"),
            content_md: row.get("content_md"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            tags: Vec::new(),
        });

        if let Ok(tag_id) = row.try_get::<i32, _>("tag_id") {
            entry.tags.push(tag_id);
        }
    }

    let mut out: Vec<Article> = map.into_values().collect();
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out
}


/// -----------------------------
/// 获取全部文章
/// -----------------------------
pub async fn get_all_articles(pool: &PgPool) -> Result<Vec<Article>, Error> {
    let rows = sqlx::query(
        r#"
        SELECT a.id,a.title,a.content_md,a.created_at,a.updated_at,
               at.tag_id
        FROM articles a
        LEFT JOIN article_tags at ON a.id = at.article_id
        ORDER BY a.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(merge_articles(rows))
}


/// -----------------------------
/// 获取全部标签
/// -----------------------------
pub async fn get_all_tags(pool: &PgPool) -> Result<Vec<Tag>, Error> {
    sqlx::query_as!(
        Tag,
        "SELECT id,name FROM tags"
    )
    .fetch_all(pool)
    .await
}


/// -----------------------------
/// 根据 tag 查文章
/// -----------------------------
pub async fn get_articles_by_tag(pool: &PgPool, tag_id: i32)
    -> Result<Vec<Article>, Error>
{
    let rows = sqlx::query(
        r#"
        SELECT a.id,a.title,a.content_md,a.created_at,a.updated_at,
               at.tag_id
        FROM articles a
        JOIN article_tags at ON a.id = at.article_id
        WHERE at.tag_id=$1
        ORDER BY a.created_at DESC
        "#
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await?;

    Ok(merge_articles(rows))
}


/// -----------------------------
/// 根据 id 查文章
/// -----------------------------
pub async fn get_article_by_id(id: i32, pool: &PgPool)
    -> Result<Article, Error>
{
    let rows = sqlx::query(
        r#"
        SELECT a.id,a.title,a.content_md,a.created_at,a.updated_at,
               at.tag_id
        FROM articles a
        LEFT JOIN article_tags at ON a.id = at.article_id
        WHERE a.id=$1
        "#
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Err(Error::RowNotFound);
    }

    Ok(merge_articles(rows).remove(0))
}


/// -----------------------------
/// 最新文章
/// -----------------------------
pub async fn get_recent_articles(pool: &PgPool, limit: i64)
    -> Result<Vec<RecentArticleView>, Error>
{
    sqlx::query_as!(
        RecentArticleView,
        r#"
        SELECT id,title,created_at
        FROM articles
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
}


/// -----------------------------
/// 热门标签
/// -----------------------------
pub async fn get_popular_tags(
    pool: &PgPool,
    limit: i64
) -> Result<Vec<TagWithCount>, Error>
{
    sqlx::query_as!(
        TagWithCount,
        r#"
        SELECT
            t.id,
            t.name,
            COALESCE(COUNT(at.article_id), 0) AS article_count
        FROM tags t
        LEFT JOIN article_tags at ON t.id = at.tag_id
        GROUP BY t.id
        ORDER BY article_count DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
}


/// -----------------------------
/// 站点统计
/// -----------------------------
pub async fn get_blog_stats(pool: &PgPool)
    -> Result<BlogStats, sqlx::Error>
{
    let article_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM articles"
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let tag_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tags"
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let info: SiteInfo =
        sqlx::query_as!(
            SiteInfo,
            "SELECT created_at, visit_count FROM site_info LIMIT 1"
        )
        .fetch_one(pool)
        .await?;

    let today = chrono::Local::now().date_naive();

    let days_running =
        (today - info.created_at.date()).num_days();

    Ok(BlogStats {
        article_count,
        tag_count,
        days_running,
        visit_count: info.visit_count,
    })
}

/// -----------------------------
/// 创建连接池
/// -----------------------------
pub async fn create_pool(database_url: &str)
    -> Result<PgPool, sqlx::Error>
{
    use sqlx::postgres::PgPoolOptions;

    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}


/// -----------------------------
/// 根据文章获取标签
/// -----------------------------
pub async fn get_tags_by_article_id(pool: &PgPool, article_id: i32)
    -> Result<Vec<Tag>, sqlx::Error>
{
    sqlx::query_as!(
        Tag,
        r#"
        SELECT t.id, t.name
        FROM tags t
        JOIN article_tags at ON t.id = at.tag_id
        WHERE at.article_id = $1
        "#,
        article_id
    )
    .fetch_all(pool)
    .await
}



pub async fn get_articles_grouped_by_year(pool: &PgPool) -> Result<Vec<(i32, Vec<Article>)>, sqlx::Error> {
    // 获取文章及其标签
    let rows = sqlx::query!(
        r#"
        SELECT a.id, a.title, a.content_md, a.created_at, a.updated_at,
               at.tag_id
        FROM articles a
        LEFT JOIN article_tags at ON a.id = at.article_id
        ORDER BY a.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    use std::collections::HashMap;
    let mut map: HashMap<i32, Article> = HashMap::new();

    for row in rows {
        let entry = map.entry(row.id).or_insert(Article {
            id: row.id,
            title: row.title.clone(),
            content_md: row.content_md.clone(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            tags: vec![],
        });

        if let Some(tag_id) = row.tag_id {
            entry.tags.push(tag_id);
        }
    }

    // 按年份分组
    let mut articles_by_year: HashMap<i32, Vec<Article>> = HashMap::new();

    for article in map.into_values() {
        let year = article.created_at.map(|dt| dt.year()).unwrap_or(0);
        articles_by_year.entry(year).or_default().push(article);
    }
    // 转换为排序后的向量
    let mut result: Vec<(i32, Vec<Article>)> = articles_by_year.into_iter().collect();
    result.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(result)
}

pub async fn create_article(pool: &PgPool, title: &str, content_md: &str, tag_ids: &[i32]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let rec = sqlx::query!(
        "INSERT INTO articles (title, content_md, created_at, updated_at)
         VALUES ($1, $2, NOW(), NOW())
         RETURNING id",
        title, content_md
    )
    .fetch_one(&mut *tx)
    .await?;

    let article_id = rec.id;

    for &tag_id in tag_ids {
        sqlx::query!("INSERT INTO article_tags (article_id, tag_id) VALUES ($1, $2)", article_id, tag_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn update_article(pool: &PgPool, id: i32, title: &str, content_md: &str, tag_ids: &[i32]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!("UPDATE articles SET title=$1, content_md=$2, updated_at=NOW() WHERE id=$3", title, content_md, id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM article_tags WHERE article_id=$1", id)
        .execute(&mut *tx)
        .await?;

    for &tag_id in tag_ids {
        sqlx::query!("INSERT INTO article_tags (article_id, tag_id) VALUES ($1, $2)", id, tag_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_article(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM article_tags WHERE article_id=$1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM articles WHERE id=$1", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

// 创建标签
pub async fn create_tag(pool: &PgPool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO tags (name) VALUES ($1)",
        name
    )
    .execute(pool)
    .await?;
    Ok(())
}

// 删除标签
pub async fn delete_tag(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM tags WHERE id=$1", id)
        .execute(pool)
        .await?;
    Ok(())
}

// 根据 ID 获取单个标签
pub async fn get_tag_by_id(pool: &PgPool, id: i32) -> Result<Tag, sqlx::Error> {
    sqlx::query_as!(Tag, "SELECT id, name FROM tags WHERE id=$1", id)
        .fetch_one(pool)
        .await
}

pub async fn increment_visit(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE site_info SET visit_count = visit_count + 1 WHERE id = 1"
    )
    .execute(pool)
    .await?;
    Ok(())
}

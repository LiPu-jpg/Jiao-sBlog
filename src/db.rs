use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::models::{Article, Tag};
use num_traits::cast::ToPrimitive;

// 获取所有文章
pub async fn get_all_articles(pool: &PgPool) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as!(Article, r#"
        SELECT id, title, content_md, created_at, updated_at
        FROM articles
        ORDER BY created_at DESC
    "#)
    .fetch_all(pool)
    .await
}

// 获取所有标签
pub async fn get_all_tags(pool: &PgPool) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as!(Tag, r#"
        SELECT id, name
        FROM tags
    "#)
    .fetch_all(pool)
    .await
}

// 根据标签 ID 获取所有文章
pub async fn get_articles_by_tag(pool: &PgPool, tag_id: i32) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as!(Article, r#"
        SELECT a.id, a.title, a.content_md, a.created_at, a.updated_at
        FROM articles a
        JOIN article_tags at ON a.id = at.article_id
        WHERE at.tag_id = $1
        ORDER BY a.created_at DESC
    "#, tag_id)
    .fetch_all(pool)
    .await
}

pub async fn create_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn get_article_by_id(id: i32, pool: &PgPool) -> Result<Article, sqlx::Error> {
    sqlx::query_as!(Article, r#"
        SELECT id, title, content_md, created_at, updated_at
        FROM articles
        WHERE id = $1
    "#, id)
    .fetch_one(pool)
    .await
}


// 获取按年分组的文章
pub async fn get_articles_grouped_by_year(pool: &PgPool) -> Result<Vec<(i32, Vec<Article>)>, sqlx::Error> {
    // 获取按年分组的文章
    let rows = sqlx::query!(
        r#"
        SELECT EXTRACT(YEAR FROM created_at) AS year, id, title, content_md, created_at, updated_at
        FROM articles
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    // 将查询结果按照年份进行分组
    let mut articles_by_year: Vec<(i32, Vec<Article>)> = Vec::new();
    let mut current_year = None;
    let mut current_articles = Vec::new();

    for row in rows {
        // 提取年份，并处理 BigDecimal 类型，转换为 i32
        let year = row.year.map(|year| year.to_i32().unwrap_or(0)).unwrap_or(0);

        let article = Article {
            id: row.id,
            title: row.title,
            content_md: row.content_md,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        if current_year.is_none() {
            current_year = Some(year);
        }

        // 如果当前年份与上一条记录的年份相同，继续添加文章
        if current_year == Some(year) {
            current_articles.push(article);
        } else {
            // 如果年份不同，则将当前文章列表添加到结果中
            if let Some(year) = current_year {
                articles_by_year.push((year, current_articles));
            }
            // 重置年份和文章列表
            current_year = Some(year);
            current_articles = vec![article];
        }
    }

    // 将最后一组文章添加到结果中
    if let Some(year) = current_year {
        articles_by_year.push((year, current_articles));
    }

    Ok(articles_by_year)
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

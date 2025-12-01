use sqlx::PgPool;
use crate::models::{Article, Tag};
use chrono::Datelike;
use sqlx::postgres::PgPoolOptions;

// 获取所有文章
pub async fn get_all_articles(pool: &PgPool) -> Result<Vec<Article>, sqlx::Error> {
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

    let mut result: Vec<Article> = map.into_values().collect();
    result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(result)
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

// 根据标签 ID 获取文章
pub async fn get_articles_by_tag(pool: &PgPool, tag_id: i32) -> Result<Vec<Article>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT a.id, a.title, a.content_md, a.created_at, a.updated_at,
               at.tag_id
        FROM articles a
        JOIN article_tags at ON a.id = at.article_id
        WHERE at.tag_id = $1
        ORDER BY a.created_at DESC
        "#,
        tag_id
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

    let mut result: Vec<Article> = map.into_values().collect();
    result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(result)
}

// 根据 ID 获取文章及其标签
pub async fn get_article_by_id(id: i32, pool: &PgPool) -> Result<Article, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT a.id, a.title, a.content_md, a.created_at, a.updated_at,
               at.tag_id
        FROM articles a
        LEFT JOIN article_tags at ON a.id = at.article_id
        WHERE a.id = $1
        "#,
        id
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let mut tags = vec![];
    for r in &rows {
        if let Some(tag_id) = r.tag_id {
            tags.push(tag_id);
        }
    }

    let first = &rows[0];
    Ok(Article {
        id: first.id,
        title: first.title.clone(),
        content_md: first.content_md.clone(),
        created_at: first.created_at,
        updated_at: first.updated_at,
        tags,
    })
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

    // 转成 Vec<(year, Vec<Article>)> 并排序
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

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
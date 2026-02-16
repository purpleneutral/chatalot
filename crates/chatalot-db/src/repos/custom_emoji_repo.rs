use sqlx::PgPool;
use uuid::Uuid;

use crate::models::custom_emoji::CustomEmoji;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    community_id: Uuid,
    shortcode: &str,
    file_path: &str,
    content_type: &str,
    uploaded_by: Uuid,
) -> Result<CustomEmoji, sqlx::Error> {
    sqlx::query_as::<_, CustomEmoji>(
        r#"
        INSERT INTO custom_emojis (id, community_id, shortcode, file_path, content_type, uploaded_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(community_id)
    .bind(shortcode)
    .bind(file_path)
    .bind(content_type)
    .bind(uploaded_by)
    .fetch_one(pool)
    .await
}

pub async fn list_for_community(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CustomEmoji>, sqlx::Error> {
    sqlx::query_as::<_, CustomEmoji>(
        "SELECT * FROM custom_emojis WHERE community_id = $1 ORDER BY shortcode ASC LIMIT 500",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await
}

pub async fn get_by_shortcode(
    pool: &PgPool,
    community_id: Uuid,
    shortcode: &str,
) -> Result<Option<CustomEmoji>, sqlx::Error> {
    sqlx::query_as::<_, CustomEmoji>(
        "SELECT * FROM custom_emojis WHERE community_id = $1 AND shortcode = $2",
    )
    .bind(community_id)
    .bind(shortcode)
    .fetch_optional(pool)
    .await
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CustomEmoji>, sqlx::Error> {
    sqlx::query_as::<_, CustomEmoji>("SELECT * FROM custom_emojis WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM custom_emojis WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn count_for_community(pool: &PgPool, community_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM custom_emojis WHERE community_id = $1")
        .bind(community_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

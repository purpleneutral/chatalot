use sqlx::PgPool;
use uuid::Uuid;

use crate::models::bookmark::Bookmark;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    message_id: Uuid,
    note: Option<&str>,
) -> Result<Bookmark, sqlx::Error> {
    sqlx::query_as::<_, Bookmark>(
        r#"
        INSERT INTO bookmarks (id, user_id, message_id, note)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, message_id) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(message_id)
    .bind(note)
    .fetch_one(pool)
    .await
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Bookmark>, sqlx::Error> {
    sqlx::query_as::<_, Bookmark>(
        "SELECT * FROM bookmarks WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM bookmarks WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn exists(pool: &PgPool, user_id: Uuid, message_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND message_id = $2)",
    )
    .bind(user_id)
    .bind(message_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

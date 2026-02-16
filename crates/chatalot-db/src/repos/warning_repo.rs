use sqlx::PgPool;
use uuid::Uuid;

use crate::models::warning::Warning;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    channel_id: Uuid,
    issued_by: Uuid,
    reason: &str,
) -> Result<Warning, sqlx::Error> {
    sqlx::query_as::<_, Warning>(
        r#"
        INSERT INTO warnings (id, user_id, channel_id, issued_by, reason)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(channel_id)
    .bind(issued_by)
    .bind(reason)
    .fetch_one(pool)
    .await
}

pub async fn list_for_user(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<Vec<Warning>, sqlx::Error> {
    sqlx::query_as::<_, Warning>(
        "SELECT * FROM warnings WHERE user_id = $1 AND channel_id = $2 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

pub async fn count_for_user_in_channel(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM warnings WHERE user_id = $1 AND channel_id = $2")
            .bind(user_id)
            .bind(channel_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM warnings WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

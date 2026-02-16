use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::timeout::UserTimeout;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    channel_id: Uuid,
    issued_by: Uuid,
    reason: Option<&str>,
    expires_at: DateTime<Utc>,
) -> Result<UserTimeout, sqlx::Error> {
    // Remove any existing timeout for this user+channel first
    sqlx::query("DELETE FROM user_timeouts WHERE user_id = $1 AND channel_id = $2")
        .bind(user_id)
        .bind(channel_id)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, UserTimeout>(
        r#"
        INSERT INTO user_timeouts (id, user_id, channel_id, issued_by, reason, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(channel_id)
    .bind(issued_by)
    .bind(reason)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn get_active_timeout(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<Option<UserTimeout>, sqlx::Error> {
    sqlx::query_as::<_, UserTimeout>(
        "SELECT * FROM user_timeouts WHERE user_id = $1 AND channel_id = $2 AND expires_at > NOW()",
    )
    .bind(user_id)
    .bind(channel_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_for_user(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<Vec<UserTimeout>, sqlx::Error> {
    sqlx::query_as::<_, UserTimeout>(
        "SELECT * FROM user_timeouts WHERE user_id = $1 AND channel_id = $2 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

pub async fn remove(pool: &PgPool, user_id: Uuid, channel_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_timeouts WHERE user_id = $1 AND channel_id = $2")
        .bind(user_id)
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_timeouts WHERE expires_at < NOW()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

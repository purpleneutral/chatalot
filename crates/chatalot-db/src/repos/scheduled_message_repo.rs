use sqlx::PgPool;
use uuid::Uuid;

use crate::models::scheduled_message::ScheduledMessage;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    channel_id: Uuid,
    user_id: Uuid,
    ciphertext: &str,
    nonce: &str,
    scheduled_for: chrono::DateTime<chrono::Utc>,
) -> Result<ScheduledMessage, sqlx::Error> {
    sqlx::query_as::<_, ScheduledMessage>(
        r#"
        INSERT INTO scheduled_messages (id, channel_id, user_id, ciphertext, nonce, scheduled_for)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(channel_id)
    .bind(user_id)
    .bind(ciphertext)
    .bind(nonce)
    .bind(scheduled_for)
    .fetch_one(pool)
    .await
}

pub async fn list_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ScheduledMessage>, sqlx::Error> {
    sqlx::query_as::<_, ScheduledMessage>(
        "SELECT * FROM scheduled_messages WHERE user_id = $1 ORDER BY scheduled_for ASC LIMIT 200",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_due_messages(pool: &PgPool) -> Result<Vec<ScheduledMessage>, sqlx::Error> {
    sqlx::query_as::<_, ScheduledMessage>(
        "SELECT * FROM scheduled_messages WHERE scheduled_for <= NOW() ORDER BY scheduled_for ASC LIMIT 500",
    )
    .fetch_all(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM scheduled_messages WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_by_id(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM scheduled_messages WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

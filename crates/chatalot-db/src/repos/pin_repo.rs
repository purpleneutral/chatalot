use sqlx::PgPool;
use uuid::Uuid;

use crate::models::pin::{PinnedMessage, PinnedMessageWithContent};

/// Pin a message. Returns the pin record.
pub async fn pin_message(
    pool: &PgPool,
    message_id: Uuid,
    channel_id: Uuid,
    pinned_by: Uuid,
) -> Result<PinnedMessage, sqlx::Error> {
    sqlx::query_as::<_, PinnedMessage>(
        r#"
        INSERT INTO pinned_messages (message_id, channel_id, pinned_by)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(message_id)
    .bind(channel_id)
    .bind(pinned_by)
    .fetch_one(pool)
    .await
}

/// Unpin a message. Returns true if a row was deleted.
pub async fn unpin_message(pool: &PgPool, message_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pinned_messages WHERE message_id = $1")
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List all pinned messages in a channel with full message content.
/// Ordered by pin time, most recent first. Excludes deleted messages.
pub async fn list_pinned_messages(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<Vec<PinnedMessageWithContent>, sqlx::Error> {
    sqlx::query_as::<_, PinnedMessageWithContent>(
        r#"
        SELECT m.id, m.channel_id, m.sender_id, m.ciphertext, m.nonce,
               m.message_type, m.sender_key_id, m.reply_to_id, m.edited_at,
               m.deleted_at, m.created_at,
               pm.pinned_by, pm.pinned_at
        FROM pinned_messages pm
        INNER JOIN messages m ON m.id = pm.message_id
        WHERE pm.channel_id = $1 AND m.deleted_at IS NULL
        ORDER BY pm.pinned_at DESC
        "#,
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

/// Count pins in a channel.
pub async fn count_pins(pool: &PgPool, channel_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pinned_messages WHERE channel_id = $1")
        .bind(channel_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Check if a message is pinned.
pub async fn is_pinned(pool: &PgPool, message_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM pinned_messages WHERE message_id = $1)")
            .bind(message_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

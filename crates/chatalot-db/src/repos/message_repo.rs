use sqlx::PgPool;
use uuid::Uuid;

use crate::models::message::Message;

/// Insert a new message (ciphertext — server cannot read it).
pub async fn create_message(
    pool: &PgPool,
    id: Uuid,
    channel_id: Uuid,
    sender_id: Uuid,
    ciphertext: &[u8],
    nonce: &[u8],
    message_type: &str,
    sender_key_id: Option<Uuid>,
    reply_to_id: Option<Uuid>,
) -> Result<Message, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (id, channel_id, sender_id, ciphertext, nonce, message_type, sender_key_id, reply_to_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(channel_id)
    .bind(sender_id)
    .bind(ciphertext)
    .bind(nonce)
    .bind(message_type)
    .bind(sender_key_id)
    .bind(reply_to_id)
    .fetch_one(pool)
    .await
}

/// Get messages in a channel, paginated by cursor (before a given message ID).
/// Returns messages in reverse chronological order.
pub async fn get_messages(
    pool: &PgPool,
    channel_id: Uuid,
    before: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Message>, sqlx::Error> {
    let limit = limit.min(100);

    if let Some(before_id) = before {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE channel_id = $1
              AND created_at < (SELECT created_at FROM messages WHERE id = $2)
              AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(channel_id)
        .bind(before_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE channel_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(channel_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

/// Search messages in a channel by content (plaintext search on ciphertext bytes).
/// Note: This works because messages are currently stored as UTF-8 bytes (Phase 1).
/// Will need to change when E2E encryption is enabled in Phase 2.
pub async fn search_messages(
    pool: &PgPool,
    channel_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<Message>, sqlx::Error> {
    let limit = limit.min(50);
    let pattern = format!("%{}%", query);
    sqlx::query_as::<_, Message>(
        r#"
        SELECT * FROM messages
        WHERE channel_id = $1
          AND deleted_at IS NULL
          AND convert_from(ciphertext, 'UTF8') ILIKE $2
        ORDER BY created_at DESC
        LIMIT $3
        "#,
    )
    .bind(channel_id)
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Get a single message by ID (needed for broadcast after edit/delete).
pub async fn get_message_by_id(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<Option<Message>, sqlx::Error> {
    sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_optional(pool)
        .await
}

/// Soft-delete a message.
pub async fn delete_message(
    pool: &PgPool,
    message_id: Uuid,
    sender_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE messages SET deleted_at = NOW() WHERE id = $1 AND sender_id = $2 AND deleted_at IS NULL",
    )
    .bind(message_id)
    .bind(sender_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Soft-delete a message as a moderator (no sender ownership check).
pub async fn delete_message_as_mod(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE messages SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(message_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Update message ciphertext (edit).
pub async fn edit_message(
    pool: &PgPool,
    message_id: Uuid,
    sender_id: Uuid,
    ciphertext: &[u8],
    nonce: &[u8],
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE messages SET ciphertext = $1, nonce = $2, edited_at = NOW()
        WHERE id = $3 AND sender_id = $4 AND deleted_at IS NULL
        "#,
    )
    .bind(ciphertext)
    .bind(nonce)
    .bind(message_id)
    .bind(sender_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

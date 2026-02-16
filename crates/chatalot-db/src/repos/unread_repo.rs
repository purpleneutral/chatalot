use sqlx::PgPool;
use uuid::Uuid;

use crate::models::reaction::ReadCursor;

/// Update the read cursor for a user in a channel.
pub async fn mark_read(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO read_cursors (user_id, channel_id, last_read_message_id, last_read_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id, channel_id) DO UPDATE
        SET last_read_message_id = $3, last_read_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(channel_id)
    .bind(message_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get the read cursor for a user in a channel.
pub async fn get_read_cursor(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<Option<ReadCursor>, sqlx::Error> {
    sqlx::query_as::<_, ReadCursor>(
        "SELECT * FROM read_cursors WHERE user_id = $1 AND channel_id = $2",
    )
    .bind(user_id)
    .bind(channel_id)
    .fetch_optional(pool)
    .await
}

/// Count of unread messages in a channel for a user.
pub async fn count_unread(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let cursor = get_read_cursor(pool, user_id, channel_id).await?;

    let count: (i64,) = match cursor.and_then(|c| c.last_read_message_id) {
        Some(last_read_id) => {
            sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM messages
                WHERE channel_id = $1
                  AND deleted_at IS NULL
                  AND created_at > (SELECT created_at FROM messages WHERE id = $2)
                "#,
            )
            .bind(channel_id)
            .bind(last_read_id)
            .fetch_one(pool)
            .await?
        }
        None => {
            // No cursor = all messages are unread
            sqlx::query_as(
                "SELECT COUNT(*) FROM messages WHERE channel_id = $1 AND deleted_at IS NULL",
            )
            .bind(channel_id)
            .fetch_one(pool)
            .await?
        }
    };

    Ok(count.0)
}

/// Mark all channels as read for a user (set cursor to latest message).
pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO read_cursors (user_id, channel_id, last_read_message_id, last_read_at)
        SELECT $1, cm.channel_id, latest.id, NOW()
        FROM channel_members cm
        JOIN LATERAL (
            SELECT id FROM messages
            WHERE channel_id = cm.channel_id AND deleted_at IS NULL
            ORDER BY created_at DESC LIMIT 1
        ) latest ON true
        WHERE cm.user_id = $1
        ON CONFLICT (user_id, channel_id) DO UPDATE
        SET last_read_message_id = EXCLUDED.last_read_message_id, last_read_at = NOW()
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Get unread counts for all channels a user is a member of.
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct ChannelUnreadCount {
    pub channel_id: Uuid,
    pub unread_count: i64,
}

pub async fn get_all_unread_counts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ChannelUnreadCount>, sqlx::Error> {
    sqlx::query_as::<_, ChannelUnreadCount>(
        r#"
        SELECT
            cm.channel_id,
            COUNT(m.id) AS unread_count
        FROM channel_members cm
        LEFT JOIN read_cursors rc
            ON rc.user_id = cm.user_id AND rc.channel_id = cm.channel_id
        LEFT JOIN messages m
            ON m.channel_id = cm.channel_id
            AND m.deleted_at IS NULL
            AND m.sender_id != $1
        WHERE cm.user_id = $1
          AND (
            rc.last_read_message_id IS NULL AND m.id IS NOT NULL
            OR m.created_at > (
                SELECT created_at FROM messages WHERE id = rc.last_read_message_id
            )
          )
        GROUP BY cm.channel_id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

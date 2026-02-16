use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::message::Message;

/// Optional filters for message search.
pub struct SearchFilters {
    pub sender: Option<String>,
    pub before: Option<DateTime<Utc>>,
    pub after: Option<DateTime<Utc>>,
    pub has_file: Option<bool>,
}

/// Escape ILIKE special characters to prevent wildcard injection.
fn escape_ilike(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

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
    plaintext: Option<&str>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Message, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (id, channel_id, sender_id, ciphertext, nonce, message_type, sender_key_id, reply_to_id, plaintext, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
    .bind(plaintext)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// Count messages in a channel (excluding deleted and quarantined).
pub async fn count_messages(pool: &PgPool, channel_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE channel_id = $1 AND deleted_at IS NULL AND quarantined_at IS NULL",
    )
    .bind(channel_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
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
              AND quarantined_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
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
            WHERE channel_id = $1
              AND deleted_at IS NULL
              AND quarantined_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
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

/// Search messages across all channels a user is a member of.
/// Note: This works because messages are currently stored as UTF-8 bytes (Phase 1).
pub async fn search_messages_global(
    pool: &PgPool,
    user_id: Uuid,
    query: &str,
    limit: i64,
    filters: &SearchFilters,
) -> Result<Vec<Message>, sqlx::Error> {
    let limit = limit.min(50);
    let pattern = format!("%{}%", escape_ilike(query));

    let mut sql = String::from(
        r#"SELECT m.* FROM messages m
        INNER JOIN channel_members cm ON cm.channel_id = m.channel_id AND cm.user_id = $1
        WHERE m.deleted_at IS NULL
          AND m.quarantined_at IS NULL
          AND convert_from(m.ciphertext, 'UTF8') ILIKE $2"#,
    );
    let mut param_idx = 4u32; // $3 is limit

    if filters.sender.is_some() {
        sql.push_str(&format!(
            " AND m.sender_id IN (SELECT id FROM users WHERE username ILIKE ${param_idx})"
        ));
        param_idx += 1;
    }
    if filters.after.is_some() {
        sql.push_str(&format!(" AND m.created_at >= ${param_idx}"));
        param_idx += 1;
    }
    if filters.before.is_some() {
        sql.push_str(&format!(" AND m.created_at <= ${param_idx}"));
        let _ = param_idx;
    }
    if filters.has_file == Some(true) {
        sql.push_str(" AND m.message_type = 'file'");
    }

    sql.push_str(" ORDER BY m.created_at DESC LIMIT $3");

    let mut q = sqlx::query_as::<_, Message>(&sql)
        .bind(user_id)
        .bind(pattern)
        .bind(limit);

    if let Some(ref sender) = filters.sender {
        q = q.bind(format!("%{}%", escape_ilike(sender)));
    }
    if let Some(after) = filters.after {
        q = q.bind(after);
    }
    if let Some(before) = filters.before {
        q = q.bind(before);
    }

    q.fetch_all(pool).await
}

/// Search messages in a channel by content (plaintext search on ciphertext bytes).
/// Note: This works because messages are currently stored as UTF-8 bytes (Phase 1).
/// Will need to change when E2E encryption is enabled in Phase 2.
pub async fn search_messages(
    pool: &PgPool,
    channel_id: Uuid,
    query: &str,
    limit: i64,
    filters: &SearchFilters,
) -> Result<Vec<Message>, sqlx::Error> {
    let limit = limit.min(50);
    let pattern = format!("%{}%", escape_ilike(query));

    let mut sql = String::from(
        r#"SELECT * FROM messages
        WHERE channel_id = $1
          AND deleted_at IS NULL
          AND quarantined_at IS NULL
          AND convert_from(ciphertext, 'UTF8') ILIKE $2"#,
    );
    let mut param_idx = 4u32; // $3 is limit

    if filters.sender.is_some() {
        sql.push_str(&format!(
            " AND sender_id IN (SELECT id FROM users WHERE username ILIKE ${param_idx})"
        ));
        param_idx += 1;
    }
    if filters.after.is_some() {
        sql.push_str(&format!(" AND created_at >= ${param_idx}"));
        param_idx += 1;
    }
    if filters.before.is_some() {
        sql.push_str(&format!(" AND created_at <= ${param_idx}"));
        let _ = param_idx;
    }
    if filters.has_file == Some(true) {
        sql.push_str(" AND message_type = 'file'");
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT $3");

    let mut q = sqlx::query_as::<_, Message>(&sql)
        .bind(channel_id)
        .bind(pattern)
        .bind(limit);

    if let Some(ref sender) = filters.sender {
        q = q.bind(format!("%{}%", escape_ilike(sender)));
    }
    if let Some(after) = filters.after {
        q = q.bind(after);
    }
    if let Some(before) = filters.before {
        q = q.bind(before);
    }

    q.fetch_all(pool).await
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
pub async fn delete_message_as_mod(pool: &PgPool, message_id: Uuid) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("UPDATE messages SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(message_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Hard-delete a single message (complete removal from DB).
pub async fn hard_delete_message(pool: &PgPool, message_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM messages WHERE id = $1")
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Hard-delete ALL messages from a user across all channels.
pub async fn hard_delete_user_messages(pool: &PgPool, sender_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM messages WHERE sender_id = $1")
        .bind(sender_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Hard-delete ALL messages in a channel.
pub async fn hard_delete_channel_messages(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM messages WHERE channel_id = $1")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Garbage collect: hard-delete messages soft-deleted more than N days ago.
pub async fn gc_soft_deleted(pool: &PgPool, days: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM messages WHERE deleted_at IS NOT NULL AND deleted_at < NOW() - make_interval(days => $1)",
    )
    .bind(days as i32)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Quarantine a message (hide without deleting, preserve for evidence).
pub async fn quarantine_message(
    pool: &PgPool,
    message_id: Uuid,
    quarantined_by: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE messages SET quarantined_at = NOW(), quarantined_by = $2 WHERE id = $1 AND quarantined_at IS NULL",
    )
    .bind(message_id)
    .bind(quarantined_by)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Unquarantine a message.
pub async fn unquarantine_message(pool: &PgPool, message_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE messages SET quarantined_at = NULL, quarantined_by = NULL WHERE id = $1 AND quarantined_at IS NOT NULL",
    )
    .bind(message_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete messages that have expired (TTL).
pub async fn delete_expired_messages(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM messages WHERE expires_at IS NOT NULL AND expires_at < NOW()")
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// Insert a webhook message (plaintext, no sender_id encryption).
pub async fn create_webhook_message(
    pool: &PgPool,
    id: Uuid,
    channel_id: Uuid,
    plaintext: &str,
) -> Result<Message, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (id, channel_id, ciphertext, nonce, message_type, plaintext)
        VALUES ($1, $2, '\x00', '\x00', 'webhook', $3)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(channel_id)
    .bind(plaintext)
    .fetch_one(pool)
    .await
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

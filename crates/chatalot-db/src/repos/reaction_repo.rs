use sqlx::PgPool;
use uuid::Uuid;

use crate::models::reaction::Reaction;

/// Add a reaction to a message. Returns the reaction or None if it already exists.
pub async fn add_reaction(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<Reaction, sqlx::Error> {
    sqlx::query_as::<_, Reaction>(
        r#"
        INSERT INTO reactions (message_id, user_id, emoji)
        VALUES ($1, $2, $3)
        ON CONFLICT (message_id, user_id, emoji) DO UPDATE SET emoji = EXCLUDED.emoji
        RETURNING *
        "#,
    )
    .bind(message_id)
    .bind(user_id)
    .bind(emoji)
    .fetch_one(pool)
    .await
}

/// Remove a reaction from a message.
pub async fn remove_reaction(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind(message_id)
    .bind(user_id)
    .bind(emoji)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Get all reactions for a message.
pub async fn get_reactions(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<Vec<Reaction>, sqlx::Error> {
    sqlx::query_as::<_, Reaction>(
        "SELECT * FROM reactions WHERE message_id = $1 ORDER BY created_at",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await
}

/// Get reaction counts grouped by emoji for a message.
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct ReactionCount {
    pub emoji: String,
    pub count: i64,
    pub user_ids: Vec<Uuid>,
}

pub async fn get_reaction_counts(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<Vec<ReactionCount>, sqlx::Error> {
    sqlx::query_as::<_, ReactionCount>(
        r#"
        SELECT emoji, COUNT(*) as count, ARRAY_AGG(user_id) as user_ids
        FROM reactions
        WHERE message_id = $1
        GROUP BY emoji
        ORDER BY MIN(created_at)
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await
}

/// Count distinct emoji types on a message.
pub async fn count_unique_reactions(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT emoji) FROM reactions WHERE message_id = $1",
    )
    .bind(message_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Get reactions for multiple messages at once, grouped by message_id and emoji.
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct MessageReactionCount {
    pub message_id: Uuid,
    pub emoji: String,
    pub count: i64,
    pub user_ids: Vec<Uuid>,
}

pub async fn get_reactions_for_messages(
    pool: &PgPool,
    message_ids: &[Uuid],
) -> Result<Vec<MessageReactionCount>, sqlx::Error> {
    sqlx::query_as::<_, MessageReactionCount>(
        r#"
        SELECT message_id, emoji, COUNT(*) as count, ARRAY_AGG(user_id) as user_ids
        FROM reactions
        WHERE message_id = ANY($1)
        GROUP BY message_id, emoji
        ORDER BY message_id, MIN(created_at)
        "#,
    )
    .bind(message_ids)
    .fetch_all(pool)
    .await
}

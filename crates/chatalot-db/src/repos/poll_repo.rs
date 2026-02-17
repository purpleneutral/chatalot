use sqlx::PgPool;
use uuid::Uuid;

use crate::models::poll::{Poll, PollVote};

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    channel_id: Uuid,
    created_by: Uuid,
    question: &str,
    options: &serde_json::Value,
    multi_select: bool,
    anonymous: bool,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Poll, sqlx::Error> {
    sqlx::query_as::<_, Poll>(
        r#"
        INSERT INTO polls (id, channel_id, created_by, question, options, multi_select, anonymous, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(channel_id)
    .bind(created_by)
    .bind(question)
    .bind(options)
    .bind(multi_select)
    .bind(anonymous)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<Poll>, sqlx::Error> {
    sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn vote(
    pool: &PgPool,
    id: Uuid,
    poll_id: Uuid,
    user_id: Uuid,
    option_index: i32,
) -> Result<Option<PollVote>, sqlx::Error> {
    sqlx::query_as::<_, PollVote>(
        r#"
        INSERT INTO poll_votes (id, poll_id, user_id, option_index)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (poll_id, user_id, option_index) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(poll_id)
    .bind(user_id)
    .bind(option_index)
    .fetch_optional(pool)
    .await
}

pub async fn remove_vote(
    pool: &PgPool,
    poll_id: Uuid,
    user_id: Uuid,
    option_index: i32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM poll_votes WHERE poll_id = $1 AND user_id = $2 AND option_index = $3",
    )
    .bind(poll_id)
    .bind(user_id)
    .bind(option_index)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_votes(pool: &PgPool, poll_id: Uuid) -> Result<Vec<PollVote>, sqlx::Error> {
    sqlx::query_as::<_, PollVote>(
        "SELECT * FROM poll_votes WHERE poll_id = $1 ORDER BY option_index ASC",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
}

/// Get all votes for a batch of polls in a single query.
pub async fn get_votes_for_polls(
    pool: &PgPool,
    poll_ids: &[Uuid],
) -> Result<Vec<PollVote>, sqlx::Error> {
    sqlx::query_as::<_, PollVote>(
        "SELECT * FROM poll_votes WHERE poll_id = ANY($1) ORDER BY poll_id, option_index ASC",
    )
    .bind(poll_ids)
    .fetch_all(pool)
    .await
}

pub async fn close(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE polls SET closed = true WHERE id = $1 AND closed = false")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_for_channel(pool: &PgPool, channel_id: Uuid) -> Result<Vec<Poll>, sqlx::Error> {
    sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE channel_id = $1 ORDER BY created_at DESC")
        .bind(channel_id)
        .fetch_all(pool)
        .await
}

/// Remove all votes for a user in a poll (for switching single-select votes).
pub async fn remove_all_votes_for_user(
    pool: &PgPool,
    poll_id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM poll_votes WHERE poll_id = $1 AND user_id = $2")
        .bind(poll_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

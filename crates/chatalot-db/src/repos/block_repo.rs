use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user_block::UserBlock;

/// Block a user.
pub async fn block_user(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_blocks (blocker_id, blocked_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Unblock a user.
pub async fn unblock_user(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2")
        .bind(blocker_id)
        .bind(blocked_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Check if blocker has blocked blocked_id.
pub async fn is_blocked(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2)",
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Check if either user has blocked the other (bidirectional check for DMs).
pub async fn is_blocked_either_way(
    pool: &PgPool,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM user_blocks WHERE (blocker_id = $1 AND blocked_id = $2) OR (blocker_id = $2 AND blocked_id = $1))",
    )
    .bind(user_a)
    .bind(user_b)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// List all users blocked by a given user.
pub async fn list_blocked_users(
    pool: &PgPool,
    blocker_id: Uuid,
) -> Result<Vec<UserBlock>, sqlx::Error> {
    sqlx::query_as::<_, UserBlock>(
        "SELECT * FROM user_blocks WHERE blocker_id = $1 ORDER BY created_at DESC",
    )
    .bind(blocker_id)
    .fetch_all(pool)
    .await
}

/// Get all user IDs that a user has blocked (for message filtering).
pub async fn get_blocked_ids(pool: &PgPool, blocker_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("SELECT blocked_id FROM user_blocks WHERE blocker_id = $1")
            .bind(blocker_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

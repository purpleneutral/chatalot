use sqlx::PgPool;
use uuid::Uuid;

use crate::models::blocked_hash::BlockedHash;

/// Check if a file hash is in the blocklist.
pub async fn is_hash_blocked(pool: &PgPool, hash: &str) -> Result<bool, sqlx::Error> {
    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM blocked_hashes WHERE hash = $1)")
            .bind(hash)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

/// Add a hash to the blocklist.
pub async fn add_blocked_hash(
    pool: &PgPool,
    id: Uuid,
    hash: &str,
    reason: Option<&str>,
    blocked_by: Uuid,
) -> Result<BlockedHash, sqlx::Error> {
    sqlx::query_as::<_, BlockedHash>(
        r#"
        INSERT INTO blocked_hashes (id, hash, reason, blocked_by)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (hash) DO UPDATE SET reason = EXCLUDED.reason
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(hash)
    .bind(reason)
    .bind(blocked_by)
    .fetch_one(pool)
    .await
}

/// Remove a hash from the blocklist.
pub async fn remove_blocked_hash(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM blocked_hashes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List blocked hashes with pagination.
pub async fn list_blocked_hashes(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<BlockedHash>, sqlx::Error> {
    sqlx::query_as::<_, BlockedHash>(
        "SELECT * FROM blocked_hashes ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Count total blocked hashes.
pub async fn count_blocked_hashes(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blocked_hashes")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

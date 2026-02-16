use sqlx::PgPool;
use uuid::Uuid;

use crate::models::sender_key::SenderKeyDistributionRow;

/// Upsert a sender key distribution for a user in a channel.
pub async fn upsert_distribution(
    pool: &PgPool,
    id: Uuid,
    channel_id: Uuid,
    user_id: Uuid,
    chain_id: i32,
    distribution: &serde_json::Value,
) -> Result<SenderKeyDistributionRow, sqlx::Error> {
    sqlx::query_as::<_, SenderKeyDistributionRow>(
        r#"
        INSERT INTO sender_key_distributions (id, channel_id, user_id, chain_id, distribution)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (channel_id, user_id) DO UPDATE
            SET id = EXCLUDED.id,
                chain_id = EXCLUDED.chain_id,
                distribution = EXCLUDED.distribution,
                created_at = NOW()
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(channel_id)
    .bind(user_id)
    .bind(chain_id)
    .bind(distribution)
    .fetch_one(pool)
    .await
}

/// Fetch all sender key distributions for a channel.
pub async fn get_channel_distributions(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<Vec<SenderKeyDistributionRow>, sqlx::Error> {
    sqlx::query_as::<_, SenderKeyDistributionRow>(
        "SELECT * FROM sender_key_distributions WHERE channel_id = $1 ORDER BY created_at ASC",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

/// Delete a user's sender key distribution in a channel (on member removal).
pub async fn delete_distribution(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM sender_key_distributions WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete a user's sender key distributions across multiple channels at once.
pub async fn delete_distributions_for_channels(
    pool: &PgPool,
    channel_ids: &[Uuid],
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    if channel_ids.is_empty() {
        return Ok(0);
    }
    let result = sqlx::query(
        "DELETE FROM sender_key_distributions WHERE channel_id = ANY($1) AND user_id = $2",
    )
    .bind(channel_ids)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Delete ALL sender key distributions for a channel (full rotation).
pub async fn delete_all_distributions(pool: &PgPool, channel_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM sender_key_distributions WHERE channel_id = $1")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

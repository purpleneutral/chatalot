use sqlx::PgPool;
use uuid::Uuid;

use crate::models::channel::{Channel, ChannelType};
use crate::models::file::DmPair;
use crate::models::user::User;

/// Find an existing DM channel between two users.
pub async fn find_dm_channel(
    pool: &PgPool,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<Option<DmPair>, sqlx::Error> {
    let (a, b) = if user_a < user_b {
        (user_a, user_b)
    } else {
        (user_b, user_a)
    };

    sqlx::query_as::<_, DmPair>("SELECT * FROM dm_pairs WHERE user_a = $1 AND user_b = $2")
        .bind(a)
        .bind(b)
        .fetch_optional(pool)
        .await
}

/// Create a DM channel between two users, or return the existing one.
pub async fn get_or_create_dm(
    pool: &PgPool,
    channel_id: Uuid,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<Channel, sqlx::Error> {
    let (a, b) = if user_a < user_b {
        (user_a, user_b)
    } else {
        (user_b, user_a)
    };

    // Check if DM already exists
    if let Some(existing) = find_dm_channel(pool, a, b).await? {
        let channel = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = $1")
            .bind(existing.channel_id)
            .fetch_one(pool)
            .await?;
        return Ok(channel);
    }

    // Create new DM channel + memberships + dm_pair in a transaction
    let mut tx = pool.begin().await?;

    let channel = sqlx::query_as::<_, Channel>(
        r#"
        INSERT INTO channels (id, name, channel_type, created_by)
        VALUES ($1, NULL, $2, $3)
        RETURNING *
        "#,
    )
    .bind(channel_id)
    .bind(ChannelType::Dm)
    .bind(a)
    .fetch_one(&mut *tx)
    .await?;

    // Add both users as members
    sqlx::query(
        "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member'), ($1, $3, 'member')",
    )
    .bind(channel_id)
    .bind(a)
    .bind(b)
    .execute(&mut *tx)
    .await?;

    // Record the DM pair
    sqlx::query("INSERT INTO dm_pairs (user_a, user_b, channel_id) VALUES ($1, $2, $3)")
        .bind(a)
        .bind(b)
        .bind(channel_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(channel)
}

/// List all DM channels for a user, with the other user's info.
/// Uses batch queries instead of per-pair lookups to avoid N+1.
pub async fn list_user_dms(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<(Channel, User)>, sqlx::Error> {
    let pairs = sqlx::query_as::<_, DmPair>(
        "SELECT * FROM dm_pairs WHERE user_a = $1 OR user_b = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    if pairs.is_empty() {
        return Ok(Vec::new());
    }

    // Collect all channel IDs and other user IDs
    let channel_ids: Vec<Uuid> = pairs.iter().map(|p| p.channel_id).collect();
    let other_user_ids: Vec<Uuid> = pairs
        .iter()
        .map(|p| if p.user_a == user_id { p.user_b } else { p.user_a })
        .collect();

    // Batch-fetch channels and users
    let channels = sqlx::query_as::<_, Channel>(
        "SELECT * FROM channels WHERE id = ANY($1)",
    )
    .bind(&channel_ids)
    .fetch_all(pool)
    .await?;

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = ANY($1)",
    )
    .bind(&other_user_ids)
    .fetch_all(pool)
    .await?;

    // Index by ID for fast lookup
    let channel_map: std::collections::HashMap<Uuid, Channel> =
        channels.into_iter().map(|c| (c.id, c)).collect();
    let user_map: std::collections::HashMap<Uuid, User> =
        users.into_iter().map(|u| (u.id, u)).collect();

    // Reassemble in original order
    let mut results = Vec::with_capacity(pairs.len());
    for (pair, other_id) in pairs.iter().zip(other_user_ids.iter()) {
        if let (Some(channel), Some(user)) = (channel_map.get(&pair.channel_id), user_map.get(other_id)) {
            results.push((channel.clone(), user.clone()));
        }
    }

    Ok(results)
}

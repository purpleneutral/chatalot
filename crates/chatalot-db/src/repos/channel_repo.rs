use sqlx::PgPool;
use uuid::Uuid;

use chrono::{DateTime, Utc};

use crate::models::channel::{Channel, ChannelMember, ChannelMemberInfo, ChannelType};

/// Create a new channel and add the creator as owner.
pub async fn create_channel(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    channel_type: ChannelType,
    topic: Option<&str>,
    created_by: Uuid,
    group_id: Option<Uuid>,
    discoverable: bool,
) -> Result<Channel, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let channel = sqlx::query_as::<_, Channel>(
        r#"
        INSERT INTO channels (id, name, channel_type, topic, created_by, group_id, discoverable)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(channel_type)
    .bind(topic)
    .bind(created_by)
    .bind(group_id)
    .bind(discoverable)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role)
        VALUES ($1, $2, 'owner')
        "#,
    )
    .bind(id)
    .bind(created_by)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(channel)
}

/// List all channels a user is a member of.
pub async fn list_user_channels(pool: &PgPool, user_id: Uuid) -> Result<Vec<Channel>, sqlx::Error> {
    sqlx::query_as::<_, Channel>(
        r#"
        SELECT c.* FROM channels c
        INNER JOIN channel_members cm ON c.id = cm.channel_id
        WHERE cm.user_id = $1
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Get a channel by ID.
pub async fn get_channel(pool: &PgPool, id: Uuid) -> Result<Option<Channel>, sqlx::Error> {
    sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Check if a channel belongs to a community (via its group).
pub async fn channel_belongs_to_community(
    pool: &PgPool,
    channel_id: Uuid,
    community_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM channels c
            JOIN groups g ON c.group_id = g.id
            WHERE c.id = $1 AND g.community_id = $2
        )"#,
    )
    .bind(channel_id)
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Check if a user is a member of a channel.
pub async fn is_member(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Add a user to a channel.
pub async fn join_channel(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role)
        VALUES ($1, $2, 'member')
        ON CONFLICT (channel_id, user_id) DO NOTHING
        "#,
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Add multiple users to a channel at once.
pub async fn join_channel_batch(
    pool: &PgPool,
    channel_id: Uuid,
    user_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(());
    }
    sqlx::query(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role)
        SELECT $1, unnest($2::uuid[]), 'member'
        ON CONFLICT (channel_id, user_id) DO NOTHING
        "#,
    )
    .bind(channel_id)
    .bind(user_ids)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a user from a channel.
pub async fn leave_channel(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2")
        .bind(channel_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// List members of a channel.
pub async fn list_members(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<Vec<ChannelMember>, sqlx::Error> {
    sqlx::query_as::<_, ChannelMember>(
        "SELECT * FROM channel_members WHERE channel_id = $1 ORDER BY joined_at ASC",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

/// List members of a channel with user profile info, ordered by role rank.
pub async fn list_members_with_users(
    pool: &PgPool,
    channel_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ChannelMemberInfo>, sqlx::Error> {
    sqlx::query_as::<_, ChannelMemberInfo>(
        r#"
        SELECT cm.channel_id, cm.user_id, cm.role, cm.joined_at,
               u.username, u.display_name, u.avatar_url
        FROM channel_members cm
        INNER JOIN users u ON u.id = cm.user_id
        WHERE cm.channel_id = $1
        ORDER BY
            CASE cm.role
                WHEN 'owner' THEN 0
                WHEN 'admin' THEN 1
                WHEN 'moderator' THEN 2
                ELSE 3
            END,
            cm.joined_at ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(channel_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// List members of a discoverable channel: explicit channel members + group members
/// who haven't explicitly joined. Channel members keep their stored role; group-only
/// members appear as "member".
pub async fn list_discoverable_channel_members(
    pool: &PgPool,
    channel_id: Uuid,
    group_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ChannelMemberInfo>, sqlx::Error> {
    sqlx::query_as::<_, ChannelMemberInfo>(
        r#"
        SELECT * FROM (
            -- Explicit channel members (keep their stored role)
            SELECT cm.channel_id, cm.user_id, cm.role, cm.joined_at,
                   u.username, u.display_name, u.avatar_url
            FROM channel_members cm
            INNER JOIN users u ON u.id = cm.user_id
            WHERE cm.channel_id = $1

            UNION ALL

            -- Group members who haven't explicitly joined (shown as "member")
            SELECT $1 AS channel_id, gm.user_id, 'member' AS role, gm.joined_at,
                   u.username, u.display_name, u.avatar_url
            FROM group_members gm
            INNER JOIN users u ON u.id = gm.user_id
            WHERE gm.group_id = $2
              AND NOT EXISTS (
                  SELECT 1 FROM channel_members cm2
                  WHERE cm2.channel_id = $1 AND cm2.user_id = gm.user_id
              )
        ) combined
        ORDER BY
            CASE role
                WHEN 'owner' THEN 0
                WHEN 'admin' THEN 1
                WHEN 'moderator' THEN 2
                ELSE 3
            END,
            joined_at ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(channel_id)
    .bind(group_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get a member's role in a channel.
pub async fn get_member_role(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT role FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

/// Update a member's role. Returns true if the row was updated.
pub async fn update_member_role(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
    new_role: &str,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("UPDATE channel_members SET role = $1 WHERE channel_id = $2 AND user_id = $3")
            .bind(new_role)
            .bind(channel_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Ban a user from a channel (removes membership and records the ban).
pub async fn ban_user(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
    banned_by: Uuid,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO channel_bans (channel_id, user_id, banned_by, reason)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (channel_id, user_id) DO NOTHING
        "#,
    )
    .bind(channel_id)
    .bind(user_id)
    .bind(banned_by)
    .bind(reason)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2")
        .bind(channel_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Remove a ban from a user.
pub async fn unban_user(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM channel_bans WHERE channel_id = $1 AND user_id = $2")
        .bind(channel_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Update a channel's name, topic, read_only, slow_mode_seconds, message_ttl_seconds, discoverable, archived, and/or voice_background.
pub async fn update_channel(
    pool: &PgPool,
    channel_id: Uuid,
    name: Option<&str>,
    topic: Option<&str>,
    read_only: Option<bool>,
    slow_mode_seconds: Option<i32>,
    message_ttl_seconds: Option<Option<i32>>,
    discoverable: Option<bool>,
    archived: Option<bool>,
    voice_background: Option<&str>,
) -> Result<Option<Channel>, sqlx::Error> {
    sqlx::query_as::<_, Channel>(
        r#"
        UPDATE channels
        SET name = COALESCE($2, name),
            topic = CASE WHEN $3 IS NULL THEN topic WHEN $3 = '' THEN NULL ELSE $3 END,
            read_only = COALESCE($4, read_only),
            slow_mode_seconds = COALESCE($5, slow_mode_seconds),
            message_ttl_seconds = COALESCE($6, message_ttl_seconds),
            discoverable = COALESCE($7, discoverable),
            archived = COALESCE($8, archived),
            voice_background = CASE WHEN $9 IS NULL THEN voice_background WHEN $9 = '' THEN NULL ELSE $9 END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(channel_id)
    .bind(name)
    .bind(topic)
    .bind(read_only)
    .bind(slow_mode_seconds)
    .bind(message_ttl_seconds)
    .bind(discoverable)
    .bind(archived)
    .bind(voice_background)
    .fetch_optional(pool)
    .await
}

/// Delete a channel.
pub async fn delete_channel(pool: &PgPool, channel_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM channels WHERE id = $1")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Transfer channel ownership: new owner becomes "owner", old owner becomes "admin".
pub async fn transfer_ownership(
    pool: &PgPool,
    channel_id: Uuid,
    old_owner_id: Uuid,
    new_owner_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Demote old owner to admin
    sqlx::query("UPDATE channel_members SET role = 'admin' WHERE channel_id = $1 AND user_id = $2")
        .bind(channel_id)
        .bind(old_owner_id)
        .execute(&mut *tx)
        .await?;

    // Promote new owner
    sqlx::query("UPDATE channel_members SET role = 'owner' WHERE channel_id = $1 AND user_id = $2")
        .bind(channel_id)
        .bind(new_owner_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Check if a user is banned from a channel.
pub async fn is_banned(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM channel_bans WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Get the last time a user sent a message in a channel (for slow mode).
pub async fn get_slowmode_last_sent(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let row: Option<(DateTime<Utc>,)> = sqlx::query_as(
        "SELECT last_sent FROM channel_slowmode_tracker WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

/// Update (upsert) the slow mode tracker for a user in a channel.
pub async fn update_slowmode_last_sent(
    pool: &PgPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO channel_slowmode_tracker (channel_id, user_id, last_sent)
        VALUES ($1, $2, NOW())
        ON CONFLICT (channel_id, user_id)
        DO UPDATE SET last_sent = NOW()
        "#,
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

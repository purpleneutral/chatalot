use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::community::{
    Community, CommunityBanInfo, CommunityInvite, CommunityMemberInfo,
};
use crate::models::user::User;

// ── CRUD ──

pub async fn create_community(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    description: Option<&str>,
    icon_url: Option<&str>,
    owner_id: Uuid,
) -> Result<Community, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let community = sqlx::query_as::<_, Community>(
        r#"
        INSERT INTO communities (id, name, description, icon_url, owner_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(icon_url)
    .bind(owner_id)
    .fetch_one(&mut *tx)
    .await?;

    // Add creator as owner member
    sqlx::query(
        r#"
        INSERT INTO community_members (community_id, user_id, role)
        VALUES ($1, $2, 'owner')
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(community)
}

pub async fn get_community(pool: &PgPool, id: Uuid) -> Result<Option<Community>, sqlx::Error> {
    sqlx::query_as::<_, Community>("SELECT * FROM communities WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_community(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    icon_url: Option<&str>,
) -> Result<Option<Community>, sqlx::Error> {
    sqlx::query_as::<_, Community>(
        r#"
        UPDATE communities
        SET name = COALESCE($2, name),
            description = COALESCE($3, description),
            icon_url = COALESCE($4, icon_url),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(icon_url)
    .fetch_optional(pool)
    .await
}

pub async fn delete_community(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM communities WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_user_communities(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Community>, sqlx::Error> {
    sqlx::query_as::<_, Community>(
        r#"
        SELECT c.* FROM communities c
        INNER JOIN community_members cm ON c.id = cm.community_id
        WHERE cm.user_id = $1
        ORDER BY c.name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

// ── Membership ──

pub async fn is_community_member(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM community_members WHERE community_id = $1 AND user_id = $2)",
    )
    .bind(community_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn get_community_member_role(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM community_members WHERE community_id = $1 AND user_id = $2",
    )
    .bind(community_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

pub async fn join_community(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO community_members (community_id, user_id, role)
        VALUES ($1, $2, 'member')
        ON CONFLICT (community_id, user_id) DO NOTHING
        "#,
    )
    .bind(community_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Leave a community — removes from community and all its groups/channels.
pub async fn leave_community(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Remove from all channels in groups belonging to this community
    sqlx::query(
        r#"
        DELETE FROM channel_members
        WHERE user_id = $2
          AND channel_id IN (
            SELECT c.id FROM channels c
            INNER JOIN groups g ON c.group_id = g.id
            WHERE g.community_id = $1
          )
        "#,
    )
    .bind(community_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Remove from all groups in this community
    sqlx::query(
        r#"
        DELETE FROM group_members
        WHERE user_id = $2
          AND group_id IN (SELECT id FROM groups WHERE community_id = $1)
        "#,
    )
    .bind(community_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Remove from community
    sqlx::query("DELETE FROM community_members WHERE community_id = $1 AND user_id = $2")
        .bind(community_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn list_community_members(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CommunityMemberInfo>, sqlx::Error> {
    sqlx::query_as::<_, CommunityMemberInfo>(
        r#"
        SELECT cm.community_id, cm.user_id, cm.role, cm.nickname, cm.joined_at,
               u.username, u.display_name, u.avatar_url
        FROM community_members cm
        INNER JOIN users u ON u.id = cm.user_id
        WHERE cm.community_id = $1
        ORDER BY
            CASE cm.role
                WHEN 'owner' THEN 0
                WHEN 'admin' THEN 1
                WHEN 'moderator' THEN 2
                ELSE 3
            END,
            cm.joined_at ASC
        "#,
    )
    .bind(community_id)
    .fetch_all(pool)
    .await
}

/// List just the user IDs of all members in a community.
pub async fn list_community_member_user_ids(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM community_members WHERE community_id = $1")
            .bind(community_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn get_community_member_count(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM community_members WHERE community_id = $1")
            .bind(community_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

pub async fn set_community_member_role(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE community_members SET role = $3 WHERE community_id = $1 AND user_id = $2",
    )
    .bind(community_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn set_community_nickname(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
    nickname: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE community_members SET nickname = $3 WHERE community_id = $1 AND user_id = $2",
    )
    .bind(community_id)
    .bind(user_id)
    .bind(nickname)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn transfer_community_ownership(
    pool: &PgPool,
    community_id: Uuid,
    old_owner_id: Uuid,
    new_owner_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE communities SET owner_id = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_owner_id)
        .bind(community_id)
        .execute(&mut *tx)
        .await?;

    // Demote old owner to admin
    sqlx::query(
        "UPDATE community_members SET role = 'admin' WHERE community_id = $1 AND user_id = $2",
    )
    .bind(community_id)
    .bind(old_owner_id)
    .execute(&mut *tx)
    .await?;

    // Promote new owner
    sqlx::query(
        "UPDATE community_members SET role = 'owner' WHERE community_id = $1 AND user_id = $2",
    )
    .bind(community_id)
    .bind(new_owner_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

// ── Bans ──

pub async fn ban_from_community(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
    banned_by: Uuid,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Insert ban record
    sqlx::query(
        r#"
        INSERT INTO community_bans (community_id, user_id, banned_by, reason)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (community_id, user_id) DO NOTHING
        "#,
    )
    .bind(community_id)
    .bind(user_id)
    .bind(banned_by)
    .bind(reason)
    .execute(&mut *tx)
    .await?;

    // Remove from all channels in groups belonging to this community
    sqlx::query(
        r#"
        DELETE FROM channel_members
        WHERE user_id = $2
          AND channel_id IN (
            SELECT c.id FROM channels c
            INNER JOIN groups g ON c.group_id = g.id
            WHERE g.community_id = $1
          )
        "#,
    )
    .bind(community_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Remove from all groups in this community
    sqlx::query(
        r#"
        DELETE FROM group_members
        WHERE user_id = $2
          AND group_id IN (SELECT id FROM groups WHERE community_id = $1)
        "#,
    )
    .bind(community_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Remove from community
    sqlx::query("DELETE FROM community_members WHERE community_id = $1 AND user_id = $2")
        .bind(community_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn unban_from_community(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM community_bans WHERE community_id = $1 AND user_id = $2")
            .bind(community_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn is_banned_from_community(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM community_bans WHERE community_id = $1 AND user_id = $2)",
    )
    .bind(community_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn list_community_bans(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CommunityBanInfo>, sqlx::Error> {
    sqlx::query_as::<_, CommunityBanInfo>(
        r#"SELECT cb.user_id, u.username, u.display_name, cb.reason, cb.created_at
        FROM community_bans cb
        INNER JOIN users u ON u.id = cb.user_id
        WHERE cb.community_id = $1
        ORDER BY cb.created_at DESC"#,
    )
    .bind(community_id)
    .fetch_all(pool)
    .await
}

// ── Invites ──

pub async fn create_community_invite(
    pool: &PgPool,
    id: Uuid,
    community_id: Uuid,
    code: &str,
    created_by: Uuid,
    max_uses: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<CommunityInvite, sqlx::Error> {
    sqlx::query_as::<_, CommunityInvite>(
        r#"
        INSERT INTO community_invites (id, community_id, code, created_by, max_uses, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(community_id)
    .bind(code)
    .bind(created_by)
    .bind(max_uses)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn list_community_invites(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CommunityInvite>, sqlx::Error> {
    sqlx::query_as::<_, CommunityInvite>(
        "SELECT * FROM community_invites WHERE community_id = $1 ORDER BY created_at DESC",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await
}

pub async fn get_community_invite_by_code(
    pool: &PgPool,
    code: &str,
) -> Result<Option<CommunityInvite>, sqlx::Error> {
    sqlx::query_as::<_, CommunityInvite>(
        "SELECT * FROM community_invites WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn delete_community_invite(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM community_invites WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_community_invite_usage(
    pool: &PgPool,
    invite_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE community_invites
        SET used_count = used_count + 1
        WHERE id = $1
          AND (max_uses IS NULL OR used_count < max_uses)
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
    )
    .bind(invite_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

// ── Shared Community Checks (for DM/user visibility scoping) ──

/// Check if two users share at least one community.
pub async fn shares_community(
    pool: &PgPool,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM community_members cm1
            INNER JOIN community_members cm2 ON cm1.community_id = cm2.community_id
            WHERE cm1.user_id = $1 AND cm2.user_id = $2
        )"#,
    )
    .bind(user_a)
    .bind(user_b)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Search users who share at least one community with the given user.
pub async fn search_visible_users(
    pool: &PgPool,
    user_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"SELECT DISTINCT u.* FROM users u
        INNER JOIN community_members cm1 ON cm1.user_id = u.id
        INNER JOIN community_members cm2 ON cm2.community_id = cm1.community_id
        WHERE cm2.user_id = $1
          AND u.id != $1
          AND (u.username ILIKE $2 OR u.display_name ILIKE $2)
        LIMIT $3"#,
    )
    .bind(user_id)
    .bind(format!("%{query}%"))
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Check if a user owns any communities (blocks account deletion).
pub async fn user_owns_communities(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM communities WHERE owner_id = $1)")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

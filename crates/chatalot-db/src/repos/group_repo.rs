use sqlx::PgPool;
use uuid::Uuid;

use crate::models::channel::Channel;
use crate::models::group::{Group, GroupMemberInfo};

/// Create a new group and add the creator as owner.
/// If `assigned_member_id` is provided, the assigned member becomes the group owner
/// in group_members instead of the creator (personal group).
pub async fn create_group(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    description: Option<&str>,
    owner_id: Uuid,
    community_id: Uuid,
    visibility: &str,
    assigned_member_id: Option<Uuid>,
) -> Result<Group, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let group = sqlx::query_as::<_, Group>(
        r#"
        INSERT INTO groups (id, name, description, owner_id, community_id, visibility, assigned_member_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(owner_id)
    .bind(community_id)
    .bind(visibility)
    .bind(assigned_member_id)
    .fetch_one(&mut *tx)
    .await?;

    // For personal groups, the assigned member is the group owner;
    // for regular groups, the creator is the owner.
    let member_owner_id = assigned_member_id.unwrap_or(owner_id);
    sqlx::query(
        r#"
        INSERT INTO group_members (group_id, user_id, role)
        VALUES ($1, $2, 'owner')
        "#,
    )
    .bind(id)
    .bind(member_owner_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(group)
}

/// List all groups a user is a member of.
pub async fn list_user_groups(pool: &PgPool, user_id: Uuid) -> Result<Vec<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>(
        r#"
        SELECT g.* FROM groups g
        INNER JOIN group_members gm ON g.id = gm.group_id
        WHERE gm.user_id = $1
        ORDER BY g.name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Get a group by ID.
pub async fn get_group(pool: &PgPool, id: Uuid) -> Result<Option<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Update a group's settings.
pub async fn update_group(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    visibility: Option<&str>,
    discoverable: Option<bool>,
    allow_invites: Option<bool>,
    icon_url: Option<&str>,
    banner_url: Option<&str>,
    accent_color: Option<&str>,
) -> Result<Option<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>(
        r#"
        UPDATE groups
        SET name = COALESCE($2, name),
            description = CASE WHEN $3 IS NULL THEN description WHEN $3 = '' THEN NULL ELSE $3 END,
            visibility = COALESCE($4, visibility),
            discoverable = COALESCE($5, discoverable),
            allow_invites = COALESCE($6, allow_invites),
            icon_url = CASE WHEN $7 IS NULL THEN icon_url WHEN $7 = '' THEN NULL ELSE $7 END,
            banner_url = CASE WHEN $8 IS NULL THEN banner_url WHEN $8 = '' THEN NULL ELSE $8 END,
            accent_color = CASE WHEN $9 IS NULL THEN accent_color WHEN $9 = '' THEN NULL ELSE $9 END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(visibility)
    .bind(discoverable)
    .bind(allow_invites)
    .bind(icon_url)
    .bind(banner_url)
    .bind(accent_color)
    .fetch_optional(pool)
    .await
}

/// Delete a group (cascades to channels and members).
pub async fn delete_group(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM groups WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Join a group — adds user as member and to all group channels.
pub async fn join_group(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO group_members (group_id, user_id, role)
        VALUES ($1, $2, 'member')
        ON CONFLICT (group_id, user_id) DO NOTHING
        "#,
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Add user to all discoverable channels in this group
    sqlx::query(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role)
        SELECT c.id, $2, 'member'
        FROM channels c
        WHERE c.group_id = $1 AND c.discoverable = TRUE
        ON CONFLICT (channel_id, user_id) DO NOTHING
        "#,
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Add multiple users to a group and all its discoverable channels at once.
pub async fn join_group_batch(
    pool: &PgPool,
    group_id: Uuid,
    user_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(0);
    }
    let mut tx = pool.begin().await?;

    // Batch insert into group_members
    let result = sqlx::query(
        r#"
        INSERT INTO group_members (group_id, user_id, role)
        SELECT $1, unnest($2::uuid[]), 'member'
        ON CONFLICT (group_id, user_id) DO NOTHING
        "#,
    )
    .bind(group_id)
    .bind(user_ids)
    .execute(&mut *tx)
    .await?;
    let added = result.rows_affected();

    // Batch insert into all discoverable channels for these users
    sqlx::query(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role)
        SELECT c.id, u.uid, 'member'
        FROM channels c
        CROSS JOIN unnest($2::uuid[]) AS u(uid)
        WHERE c.group_id = $1 AND c.discoverable = TRUE
        ON CONFLICT (channel_id, user_id) DO NOTHING
        "#,
    )
    .bind(group_id)
    .bind(user_ids)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(added)
}

/// Leave a group — removes from group and all its channels.
pub async fn leave_group(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
        .bind(group_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // Remove from all channels in this group
    sqlx::query(
        r#"
        DELETE FROM channel_members
        WHERE user_id = $2
          AND channel_id IN (SELECT id FROM channels WHERE group_id = $1)
        "#,
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// List members of a group with user profile info.
pub async fn list_group_members(
    pool: &PgPool,
    group_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<GroupMemberInfo>, sqlx::Error> {
    sqlx::query_as::<_, GroupMemberInfo>(
        r#"
        SELECT gm.group_id, gm.user_id, gm.role, gm.joined_at,
               u.username, u.display_name, u.avatar_url
        FROM group_members gm
        INNER JOIN users u ON u.id = gm.user_id
        WHERE gm.group_id = $1
        ORDER BY
            CASE gm.role
                WHEN 'owner' THEN 0
                WHEN 'admin' THEN 1
                ELSE 2
            END,
            gm.joined_at ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(group_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Check if a user owns any groups (blocks account deletion).
pub async fn user_owns_groups(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM groups WHERE owner_id = $1)")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Transfer group ownership: update owner_id, new owner gets "owner" role, old owner becomes "admin".
pub async fn transfer_ownership(
    pool: &PgPool,
    group_id: Uuid,
    old_owner_id: Uuid,
    new_owner_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Update group owner_id
    sqlx::query("UPDATE groups SET owner_id = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_owner_id)
        .bind(group_id)
        .execute(&mut *tx)
        .await?;

    // Demote old owner to admin
    sqlx::query("UPDATE group_members SET role = 'admin' WHERE group_id = $1 AND user_id = $2")
        .bind(group_id)
        .bind(old_owner_id)
        .execute(&mut *tx)
        .await?;

    // Promote new owner
    sqlx::query("UPDATE group_members SET role = 'owner' WHERE group_id = $1 AND user_id = $2")
        .bind(group_id)
        .bind(new_owner_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Get a member's role in a group.
pub async fn get_member_role(
    pool: &PgPool,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT role FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

/// Check if a user is a member of a group.
pub async fn is_member(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// List all channels in a group.
pub async fn list_group_channels(
    pool: &PgPool,
    group_id: Uuid,
) -> Result<Vec<Channel>, sqlx::Error> {
    sqlx::query_as::<_, Channel>(
        "SELECT * FROM channels WHERE group_id = $1 ORDER BY created_at ASC",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
}

/// List channels visible to a user: discoverable channels + channels they're a member of.
pub async fn list_visible_group_channels(
    pool: &PgPool,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<Channel>, sqlx::Error> {
    sqlx::query_as::<_, Channel>(
        r#"
        SELECT c.* FROM channels c
        WHERE c.group_id = $1
          AND (c.discoverable = TRUE OR EXISTS (
            SELECT 1 FROM channel_members cm WHERE cm.channel_id = c.id AND cm.user_id = $2
          ))
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Get the member count for a group.
pub async fn get_member_count(pool: &PgPool, group_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM group_members WHERE group_id = $1")
        .bind(group_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Get member counts for multiple groups in a single query.
pub async fn get_member_counts(
    pool: &PgPool,
    group_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, i64>, sqlx::Error> {
    let rows: Vec<(Uuid, i64)> = sqlx::query_as(
        "SELECT group_id, COUNT(*) FROM group_members WHERE group_id = ANY($1) GROUP BY group_id",
    )
    .bind(group_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

/// List all groups (for discovery/browsing).
pub async fn list_all_groups(pool: &PgPool) -> Result<Vec<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>("SELECT * FROM groups ORDER BY name ASC LIMIT 200")
        .fetch_all(pool)
        .await
}

/// List groups in communities the user belongs to (for scoped discovery).
/// Only shows discoverable public groups + groups the user is already a member of.
pub async fn list_discoverable_groups(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>(
        r#"SELECT g.* FROM groups g
         JOIN community_members cm ON cm.community_id = g.community_id
         WHERE cm.user_id = $1
           AND ((g.visibility = 'public' AND g.discoverable = TRUE)
                OR EXISTS (SELECT 1 FROM group_members gm WHERE gm.group_id = g.id AND gm.user_id = $1))
         ORDER BY g.name ASC
         LIMIT 200"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// List IDs of public groups in a community (for auto-joining new members).
pub async fn list_public_group_ids(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM groups WHERE community_id = $1 AND visibility = 'public'",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await
}

/// List groups in a community that the user is a member of.
pub async fn list_community_groups(
    pool: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>(
        "SELECT g.* FROM groups g \
         INNER JOIN group_members gm ON g.id = gm.group_id \
         WHERE g.community_id = $1 AND gm.user_id = $2 \
         ORDER BY g.name ASC",
    )
    .bind(community_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

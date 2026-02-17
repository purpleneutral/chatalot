use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::routing::{get, patch, post};
use axum::{Extension, Json, Router};
use sqlx::query_scalar;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use chatalot_common::api_types::{
    AcceptInviteResponse, ChannelResponse, CreateChannelRequest, CreateGroupRequest,
    CreateInviteRequest, GroupMemberResponse, GroupResponse, InviteInfoResponse, InviteResponse,
    PaginationQuery, TransferOwnershipRequest, UpdateChannelRequest, UpdateGroupRequest,
};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::models::channel::ChannelType;
use chatalot_db::models::group::Group;
use chatalot_db::repos::{channel_repo, community_repo, group_repo, invite_repo, sender_key_repo, user_repo};
use rand::Rng as _;
use sqlx::PgPool;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::permissions;

/// Get the effective role for a user in a group.
/// For regular groups, returns the group_members role.
/// For personal groups, community moderators+ get implicit 'admin' access.
async fn get_effective_group_role(
    db: &PgPool,
    group: &Group,
    user_id: Uuid,
    is_instance_owner: bool,
) -> Result<Option<String>, AppError> {
    // Check direct group membership first
    if let Some(role) = group_repo::get_member_role(db, group.id, user_id).await? {
        return Ok(Some(role));
    }

    // For personal groups, community moderators+ get admin access
    if group.assigned_member_id.is_some() {
        if is_instance_owner {
            return Ok(Some("admin".to_string()));
        }
        if let Some(community_role) =
            community_repo::get_community_member_role(db, group.community_id, user_id).await?
            && matches!(community_role.as_str(), "owner" | "admin" | "moderator")
        {
            return Ok(Some("admin".to_string()));
        }
    }

    Ok(None)
}

/// Check if a user is a community moderator+ for the group's community.
async fn is_community_moderator(
    db: &PgPool,
    community_id: Uuid,
    user_id: Uuid,
    is_instance_owner: bool,
) -> Result<bool, AppError> {
    if is_instance_owner {
        return Ok(true);
    }
    if let Some(role) = community_repo::get_community_member_role(db, community_id, user_id).await?
    {
        return Ok(matches!(role.as_str(), "owner" | "admin" | "moderator"));
    }
    Ok(false)
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/discover", get(discover_groups))
        .route(
            "/groups/{id}",
            get(get_group)
                .patch(update_group)
                .delete(delete_group_handler),
        )
        .route(
            "/groups/{id}/transfer-ownership",
            post(transfer_group_ownership),
        )
        .route("/groups/{id}/join", post(join_group))
        .route("/groups/{id}/leave", post(leave_group))
        .route("/groups/{id}/members", get(list_group_members))
        .route(
            "/groups/{id}/channels",
            get(list_group_channels).post(create_group_channel),
        )
        .route(
            "/groups/{group_id}/channels/{channel_id}",
            patch(update_group_channel).delete(delete_group_channel),
        )
        .route(
            "/groups/{id}/invites",
            get(list_invites).post(create_invite),
        )
        .route(
            "/groups/{id}/invites/{invite_id}",
            axum::routing::delete(delete_invite),
        )
        .route("/invites/{code}", get(get_invite_info))
        .route("/invites/{code}/accept", post(accept_invite))
        .route("/groups/{id}/icon", post(upload_group_icon))
        .route("/groups/{id}/banner", post(upload_group_banner))
        .route(
            "/groups/{group_id}/channels/{channel_id}/voice-background",
            post(upload_channel_voice_background),
        )
        .route("/group-assets/{filename}", get(serve_group_asset))
}

async fn create_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    let name = req.name.trim();
    if name.is_empty() || name.len() > 64 {
        return Err(AppError::Validation(
            "group name must be 1-64 characters".to_string(),
        ));
    }

    let description = req.description.as_deref().map(str::trim);
    if let Some(desc) = description
        && desc.len() > 2048
    {
        return Err(AppError::Validation(
            "description must be at most 2048 characters".to_string(),
        ));
    }

    // Verify caller is a community member — check policy for who can create groups
    let community_role =
        community_repo::get_community_member_role(&state.db, req.community_id, claims.sub)
            .await?
            .ok_or(AppError::Forbidden)?;

    let effective_role = if claims.is_owner {
        "instance_admin"
    } else {
        &community_role
    };

    // Look up the community's policy for group creation
    let community = community_repo::get_community(&state.db, req.community_id)
        .await?
        .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    if !permissions::meets_policy(effective_role, &community.who_can_create_groups) {
        return Err(AppError::Forbidden);
    }

    // Personal group: caller must be community moderator+, target must be a community member
    let is_personal = req.assigned_member_id.is_some();
    if let Some(target_id) = req.assigned_member_id {
        if !is_community_moderator(&state.db, req.community_id, claims.sub, claims.is_owner).await?
        {
            return Err(AppError::Forbidden);
        }
        if !community_repo::is_community_member(&state.db, req.community_id, target_id).await? {
            return Err(AppError::Validation(
                "assigned member must be a member of this community".to_string(),
            ));
        }
    }

    // Personal groups are forced private + non-discoverable
    let visibility = if is_personal {
        "private"
    } else {
        let v = req.visibility.as_deref().unwrap_or("public");
        if v != "public" && v != "private" {
            return Err(AppError::Validation(
                "visibility must be 'public' or 'private'".to_string(),
            ));
        }
        v
    };

    // Enforce groups-per-community limit
    let group_count: i64 = query_scalar("SELECT COUNT(*) FROM groups WHERE community_id = $1")
        .bind(req.community_id)
        .fetch_one(&state.db)
        .await?;
    if group_count >= 200 {
        return Err(AppError::Validation(
            "maximum of 200 groups per community".to_string(),
        ));
    }

    let group_id = Uuid::now_v7();
    let group = group_repo::create_group(
        &state.db,
        group_id,
        name,
        description,
        claims.sub,
        req.community_id,
        visibility,
        req.assigned_member_id,
    )
    .await?;

    // Auto-create #general text channel (created_by = assigned member or creator)
    let channel_creator = req.assigned_member_id.unwrap_or(claims.sub);
    let channel_id = Uuid::now_v7();
    channel_repo::create_channel(
        &state.db,
        channel_id,
        "general",
        ChannelType::Text,
        None,
        channel_creator,
        Some(group_id),
    )
    .await?;

    // Auto-add all community members to public groups; private/personal groups only have the owner member
    let mut member_count: i64 = 1;
    if visibility == "public" && !is_personal {
        let member_ids =
            community_repo::list_community_member_user_ids(&state.db, req.community_id).await?;
        if member_ids.len() > 5000 {
            return Err(AppError::Validation(
                "Community is too large for public group auto-join; create a private group and use invites".into(),
            ));
        }
        let other_ids: Vec<Uuid> = member_ids
            .into_iter()
            .filter(|uid| *uid != claims.sub) // Creator already added as owner
            .collect();
        let added = group_repo::join_group_batch(&state.db, group_id, &other_ids).await?;
        member_count += added as i64;
    }

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        icon_url: group.icon_url,
        banner_url: group.banner_url,
        accent_color: group.accent_color,
        owner_id: group.owner_id,
        community_id: group.community_id,
        created_at: group.created_at.to_rfc3339(),
        member_count,
        visibility: group.visibility,
        discoverable: group.discoverable,
        assigned_member_id: group.assigned_member_id,
        allow_invites: group.allow_invites,
    }))
}

async fn list_groups(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<GroupResponse>>, AppError> {
    let groups = group_repo::list_user_groups(&state.db, claims.sub).await?;
    let group_ids: Vec<Uuid> = groups.iter().map(|g| g.id).collect();
    let counts = group_repo::get_member_counts(&state.db, &group_ids).await?;
    let responses = groups
        .into_iter()
        .map(|g| {
            let count = counts.get(&g.id).copied().unwrap_or(0);
            GroupResponse {
                id: g.id,
                name: g.name,
                description: g.description,
                icon_url: g.icon_url,
                banner_url: g.banner_url,
                accent_color: g.accent_color,
                owner_id: g.owner_id,
                community_id: g.community_id,
                created_at: g.created_at.to_rfc3339(),
                member_count: count,
                visibility: g.visibility,
                discoverable: g.discoverable,
                assigned_member_id: g.assigned_member_id,
                allow_invites: g.allow_invites,
            }
        })
        .collect();
    Ok(Json(responses))
}

async fn discover_groups(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<GroupResponse>>, AppError> {
    // Instance owner sees all; everyone else only sees public groups
    // in their communities + private groups they're already in
    let groups = if claims.is_owner {
        group_repo::list_all_groups(&state.db).await?
    } else {
        group_repo::list_discoverable_groups(&state.db, claims.sub).await?
    };
    let group_ids: Vec<Uuid> = groups.iter().map(|g| g.id).collect();
    let counts = group_repo::get_member_counts(&state.db, &group_ids).await?;
    let responses = groups
        .into_iter()
        .map(|g| {
            let count = counts.get(&g.id).copied().unwrap_or(0);
            GroupResponse {
                id: g.id,
                name: g.name,
                description: g.description,
                icon_url: g.icon_url,
                banner_url: g.banner_url,
                accent_color: g.accent_color,
                owner_id: g.owner_id,
                community_id: g.community_id,
                created_at: g.created_at.to_rfc3339(),
                member_count: count,
                visibility: g.visibility,
                discoverable: g.discoverable,
                assigned_member_id: g.assigned_member_id,
                allow_invites: g.allow_invites,
            }
        })
        .collect();
    Ok(Json(responses))
}

async fn get_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<GroupResponse>, AppError> {
    let group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    // Use effective role check (allows community moderators to view personal groups)
    let _role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    let count = group_repo::get_member_count(&state.db, id).await?;

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        icon_url: group.icon_url,
        banner_url: group.banner_url,
        accent_color: group.accent_color,
        owner_id: group.owner_id,
        community_id: group.community_id,
        created_at: group.created_at.to_rfc3339(),
        member_count: count,
        visibility: group.visibility,
        discoverable: group.discoverable,
        assigned_member_id: group.assigned_member_id,
        allow_invites: group.allow_invites,
    }))
}

async fn update_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    let current_group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &current_group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    // Only community moderator+ can change allow_invites (not the assigned member)
    let allow_invites_update = if req.allow_invites.is_some() {
        if !is_community_moderator(
            &state.db,
            current_group.community_id,
            claims.sub,
            claims.is_owner,
        )
        .await?
        {
            return Err(AppError::Forbidden);
        }
        req.allow_invites
    } else {
        None
    };

    // Validate and trim inputs
    let name = req.name.as_deref().map(str::trim);
    let description = req.description.as_deref().map(str::trim);

    if let Some(n) = name
        && (n.is_empty() || n.len() > 64)
    {
        return Err(AppError::Validation(
            "group name must be 1–64 characters".to_string(),
        ));
    }

    // Validate visibility if provided
    if let Some(ref vis) = req.visibility
        && vis != "public"
        && vis != "private"
    {
        return Err(AppError::Validation(
            "visibility must be 'public' or 'private'".to_string(),
        ));
    }

    if let Some(d) = description
        && d.len() > 2048
    {
        return Err(AppError::Validation(
            "description must be at most 2048 characters".to_string(),
        ));
    }

    // Validate accent_color format if provided
    if let Some(ref color) = req.accent_color {
        static COLOR_HEX_RE: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new(r"^#[0-9a-fA-F]{3,8}$").expect("valid hex color regex"));
        if !COLOR_HEX_RE.is_match(color) {
            return Err(AppError::Validation(
                "accent_color must be a hex color (e.g. #ff0000)".to_string(),
            ));
        }
    }

    let group = group_repo::update_group(
        &state.db,
        id,
        name,
        description,
        req.visibility.as_deref(),
        req.discoverable,
        allow_invites_update,
        req.icon_url.as_deref(),
        req.banner_url.as_deref(),
        req.accent_color.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let count = group_repo::get_member_count(&state.db, id).await?;

    // Broadcast group settings change to all connected users
    state.connections.broadcast_all(ServerMessage::GroupUpdated {
        group_id: group.id,
        name: group.name.clone(),
        description: group.description.clone(),
        icon_url: group.icon_url.clone(),
        banner_url: group.banner_url.clone(),
        accent_color: group.accent_color.clone(),
        visibility: group.visibility.clone(),
    });

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        icon_url: group.icon_url,
        banner_url: group.banner_url,
        accent_color: group.accent_color,
        owner_id: group.owner_id,
        community_id: group.community_id,
        created_at: group.created_at.to_rfc3339(),
        member_count: count,
        visibility: group.visibility,
        discoverable: group.discoverable,
        assigned_member_id: group.assigned_member_id,
        allow_invites: group.allow_invites,
    }))
}

async fn delete_group_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    let group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    // For personal groups: only the original creator (owner_id) or community moderator+ can delete.
    // For regular groups: only the group owner can delete.
    if group.assigned_member_id.is_some() {
        if group.owner_id != claims.sub
            && !is_community_moderator(&state.db, group.community_id, claims.sub, claims.is_owner)
                .await?
        {
            return Err(AppError::Forbidden);
        }
    } else {
        let role = group_repo::get_member_role(&state.db, id, claims.sub)
            .await?
            .ok_or(AppError::Forbidden)?;
        if role != "owner" {
            return Err(AppError::Forbidden);
        }
    }

    group_repo::delete_group(&state.db, id).await?;

    // Broadcast group deletion to all connected users
    state.connections.broadcast_all(ServerMessage::GroupDeleted { group_id: id });

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "group_deleted",
        None,
        None,
        Some(serde_json::json!({
            "group_id": id,
            "group_name": group.name,
        })),
    )
    .await?;

    Ok(())
}

async fn transfer_group_ownership(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(group_id): Path<Uuid>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<(), AppError> {
    let group = group_repo::get_group(&state.db, group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    // Personal groups cannot have ownership transferred
    if group.assigned_member_id.is_some() {
        return Err(AppError::Validation(
            "cannot transfer ownership of a personal group".to_string(),
        ));
    }

    // Only the current owner can transfer
    let role = group_repo::get_member_role(&state.db, group_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" {
        return Err(AppError::Forbidden);
    }

    // Target must be a member of the group
    if !group_repo::is_member(&state.db, group_id, req.new_owner_id).await? {
        return Err(AppError::Validation(
            "target user must be a member of this group".to_string(),
        ));
    }

    group_repo::transfer_ownership(&state.db, group_id, claims.sub, req.new_owner_id).await?;

    Ok(())
}

async fn join_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    // Verify group exists
    let group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    // Private groups cannot be joined directly — must use an invite
    if group.visibility == "private" {
        return Err(AppError::Forbidden);
    }

    // Verify caller is a member of the group's community
    if !claims.is_owner
        && !community_repo::is_community_member(&state.db, group.community_id, claims.sub).await?
    {
        return Err(AppError::Forbidden);
    }

    group_repo::join_group(&state.db, id, claims.sub).await?;
    Ok(())
}

async fn leave_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    // Owners cannot leave — they must delete or transfer
    let role = group_repo::get_member_role(&state.db, id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role == "owner" {
        return Err(AppError::Validation(
            "owner cannot leave; transfer ownership or delete the group instead".to_string(),
        ));
    }

    group_repo::leave_group(&state.db, id, claims.sub).await?;

    // Delete leaving user's sender keys and trigger rotation for all group channels
    let channels = group_repo::list_group_channels(&state.db, id).await?;
    let channel_ids: Vec<Uuid> = channels.iter().map(|ch| ch.id).collect();
    let _ =
        sender_key_repo::delete_distributions_for_channels(&state.db, &channel_ids, claims.sub)
            .await;
    for ch in &channels {
        state.connections.broadcast_to_channel(
            ch.id,
            ServerMessage::SenderKeyRotationRequired {
                channel_id: ch.id,
                reason: "member_left".to_string(),
            },
        );
    }

    Ok(())
}

async fn list_group_members(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<GroupMemberResponse>>, AppError> {
    let group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let _role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    let limit = pagination.limit.unwrap_or(200).clamp(1, 500);
    let offset = pagination.offset.unwrap_or(0).max(0);
    let members = group_repo::list_group_members(&state.db, id, limit, offset).await?;
    Ok(Json(
        members
            .into_iter()
            .map(|m| GroupMemberResponse {
                user_id: m.user_id,
                username: m.username,
                display_name: m.display_name,
                avatar_url: m.avatar_url,
                role: m.role,
                joined_at: m.joined_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn list_group_channels(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ChannelResponse>>, AppError> {
    let group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let _role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    let channels = group_repo::list_visible_group_channels(&state.db, id, claims.sub).await?;
    Ok(Json(
        channels
            .iter()
            .map(|ch| ChannelResponse {
                id: ch.id,
                name: ch.name.clone(),
                channel_type: format!("{:?}", ch.channel_type).to_lowercase(),
                topic: ch.topic.clone(),
                created_by: ch.created_by,
                created_at: ch.created_at.to_rfc3339(),
                group_id: ch.group_id,
                read_only: ch.read_only,
                slow_mode_seconds: ch.slow_mode_seconds,
                discoverable: ch.discoverable,
                archived: ch.archived,
                voice_background: ch.voice_background.clone(),
            })
            .collect(),
    ))
}

async fn create_group_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(group_id): Path<Uuid>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<Json<ChannelResponse>, AppError> {
    let group = group_repo::get_group(&state.db, group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    let channel_type = match req.channel_type.as_str() {
        "text" => ChannelType::Text,
        "voice" => ChannelType::Voice,
        _ => return Err(AppError::Validation("invalid channel type".to_string())),
    };

    if req.name.is_empty() || req.name.len() > 64 {
        return Err(AppError::Validation(
            "channel name must be 1-64 characters".to_string(),
        ));
    }

    if let Some(ref topic) = req.topic
        && topic.len() > 512
    {
        return Err(AppError::Validation(
            "topic must be at most 512 characters".to_string(),
        ));
    }

    // Enforce channel-per-group limit
    let channel_count: i64 = query_scalar("SELECT COUNT(*) FROM channels WHERE group_id = $1")
        .bind(group_id)
        .fetch_one(&state.db)
        .await?;
    if channel_count >= 100 {
        return Err(AppError::Validation(
            "maximum of 100 channels per group".to_string(),
        ));
    }

    let channel_id = Uuid::now_v7();
    let channel = channel_repo::create_channel(
        &state.db,
        channel_id,
        &req.name,
        channel_type,
        req.topic.as_deref(),
        claims.sub,
        Some(group_id),
    )
    .await?;

    // Add all existing group members to the new channel
    let members = group_repo::list_group_members(&state.db, group_id, 10_000, 0).await?;
    let member_ids: Vec<Uuid> = members
        .iter()
        .filter(|m| m.user_id != claims.sub) // Creator is already added by create_channel
        .map(|m| m.user_id)
        .collect();
    channel_repo::join_channel_batch(&state.db, channel_id, &member_ids).await?;

    Ok(Json(ChannelResponse {
        id: channel.id,
        name: channel.name,
        channel_type: format!("{:?}", channel.channel_type).to_lowercase(),
        topic: channel.topic,
        created_by: channel.created_by,
        created_at: channel.created_at.to_rfc3339(),
        group_id: channel.group_id,
        read_only: channel.read_only,
        slow_mode_seconds: channel.slow_mode_seconds,
        discoverable: channel.discoverable,
        archived: channel.archived,
        voice_background: channel.voice_background,
    }))
}

#[derive(serde::Deserialize)]
struct GroupChannelPath {
    group_id: Uuid,
    channel_id: Uuid,
}

async fn update_group_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(path): Path<GroupChannelPath>,
    Json(req): Json<UpdateChannelRequest>,
) -> Result<Json<ChannelResponse>, AppError> {
    let group = group_repo::get_group(&state.db, path.group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    let ch_name = req.name.as_deref().map(str::trim);
    let ch_topic = req.topic.as_deref().map(str::trim);

    if let Some(n) = ch_name
        && (n.is_empty() || n.len() > 64)
    {
        return Err(AppError::Validation(
            "channel name must be 1-64 characters".to_string(),
        ));
    }

    if let Some(t) = ch_topic
        && t.len() > 512
    {
        return Err(AppError::Validation(
            "topic must be at most 512 characters".to_string(),
        ));
    }

    if let Some(sms) = req.slow_mode_seconds
        && !(0..=86400).contains(&sms)
    {
        return Err(AppError::Validation(
            "slow_mode_seconds must be between 0 and 86400".to_string(),
        ));
    }

    // Verify the channel actually belongs to this group
    let existing = channel_repo::get_channel(&state.db, path.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("channel not found".to_string()))?;
    if existing.group_id != Some(path.group_id) {
        return Err(AppError::Forbidden);
    }

    let channel = channel_repo::update_channel(
        &state.db,
        path.channel_id,
        ch_name,
        ch_topic,
        req.read_only,
        req.slow_mode_seconds,
        None,
        req.discoverable,
        req.archived,
        req.voice_background.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("channel not found".to_string()))?;

    Ok(Json(ChannelResponse {
        id: channel.id,
        name: channel.name,
        channel_type: format!("{:?}", channel.channel_type).to_lowercase(),
        topic: channel.topic,
        created_by: channel.created_by,
        created_at: channel.created_at.to_rfc3339(),
        group_id: channel.group_id,
        read_only: channel.read_only,
        slow_mode_seconds: channel.slow_mode_seconds,
        discoverable: channel.discoverable,
        archived: channel.archived,
        voice_background: channel.voice_background,
    }))
}

async fn delete_group_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(path): Path<GroupChannelPath>,
) -> Result<(), AppError> {
    let group = group_repo::get_group(&state.db, path.group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    // Verify the channel actually belongs to this group
    let existing = channel_repo::get_channel(&state.db, path.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("channel not found".to_string()))?;
    if existing.group_id != Some(path.group_id) {
        return Err(AppError::Forbidden);
    }

    channel_repo::delete_channel(&state.db, path.channel_id).await?;

    // Broadcast channel deletion to all connected users
    state.connections.broadcast_all(ServerMessage::ChannelDeleted {
        channel_id: path.channel_id,
    });

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "channel_deleted",
        None,
        None,
        Some(serde_json::json!({
            "channel_id": path.channel_id,
            "group_id": path.group_id,
        })),
    )
    .await?;

    Ok(())
}

// ── Invite endpoints ──

fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

async fn create_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(group_id): Path<Uuid>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<Json<InviteResponse>, AppError> {
    let group = group_repo::get_group(&state.db, group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    // Personal groups with allow_invites=false: only community moderator+ can create invites
    if group.assigned_member_id.is_some()
        && !group.allow_invites
        && !is_community_moderator(&state.db, group.community_id, claims.sub, claims.is_owner)
            .await?
    {
        return Err(AppError::Validation(
            "invites are disabled for this personal group".to_string(),
        ));
    }

    if let Some(h) = req.expires_in_hours
        && !(1..=8760).contains(&h)
    {
        return Err(AppError::Validation(
            "expires_in_hours must be between 1 and 8760".to_string(),
        ));
    }

    let expires_at = req
        .expires_in_hours
        .map(|h| chrono::Utc::now() + chrono::Duration::hours(h as i64));

    let invite_id = Uuid::now_v7();
    let code = generate_invite_code();

    let invite = invite_repo::create_invite(
        &state.db,
        invite_id,
        group_id,
        claims.sub,
        &code,
        req.max_uses,
        expires_at,
    )
    .await?;

    Ok(Json(InviteResponse {
        id: invite.id,
        code: invite.code,
        group_id: invite.group_id,
        max_uses: invite.max_uses,
        used_count: invite.used_count,
        expires_at: invite.expires_at.map(|t| t.to_rfc3339()),
        created_at: invite.created_at.to_rfc3339(),
    }))
}

async fn list_invites(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<InviteResponse>>, AppError> {
    let group = group_repo::get_group(&state.db, group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    let invites = invite_repo::list_group_invites(&state.db, group_id).await?;
    Ok(Json(
        invites
            .into_iter()
            .map(|i| InviteResponse {
                id: i.id,
                code: i.code,
                group_id: i.group_id,
                max_uses: i.max_uses,
                used_count: i.used_count,
                expires_at: i.expires_at.map(|t| t.to_rfc3339()),
                created_at: i.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

#[derive(serde::Deserialize)]
struct GroupInvitePath {
    id: Uuid,
    invite_id: Uuid,
}

async fn delete_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(path): Path<GroupInvitePath>,
) -> Result<(), AppError> {
    let group = group_repo::get_group(&state.db, path.id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    let deleted = invite_repo::delete_invite(&state.db, path.invite_id, path.id).await?;
    if !deleted {
        return Err(AppError::NotFound("invite not found".to_string()));
    }
    Ok(())
}

async fn get_invite_info(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<InviteInfoResponse>, AppError> {
    if code.len() > 100 {
        return Err(AppError::NotFound("invite not found".to_string()));
    }

    let invite = invite_repo::get_invite_by_code(&state.db, &code)
        .await?
        .ok_or_else(|| AppError::NotFound("invite not found".to_string()))?;

    // Check expiry
    if let Some(expires_at) = invite.expires_at
        && expires_at < chrono::Utc::now()
    {
        return Err(AppError::NotFound("invite expired".to_string()));
    }

    // Check usage
    if let Some(max_uses) = invite.max_uses
        && invite.used_count >= max_uses
    {
        return Err(AppError::NotFound("invite fully used".to_string()));
    }

    let group = group_repo::get_group(&state.db, invite.group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let count = group_repo::get_member_count(&state.db, group.id).await?;

    // Redact info for non-discoverable groups
    let (display_name, display_desc) = if group.discoverable {
        (group.name, group.description)
    } else {
        ("Private Group".to_string(), None)
    };

    Ok(Json(InviteInfoResponse {
        group_name: display_name,
        group_description: display_desc,
        member_count: count,
        code: invite.code,
    }))
}

async fn accept_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(code): Path<String>,
) -> Result<Json<AcceptInviteResponse>, AppError> {
    if code.len() > 100 {
        return Err(AppError::NotFound("invite not found".to_string()));
    }

    let invite = invite_repo::get_invite_by_code(&state.db, &code)
        .await?
        .ok_or_else(|| AppError::NotFound("invite not found".to_string()))?;

    // Check expiry
    if let Some(expires_at) = invite.expires_at
        && expires_at < chrono::Utc::now()
    {
        return Err(AppError::Validation("invite expired".to_string()));
    }

    // Check usage
    if let Some(max_uses) = invite.max_uses
        && invite.used_count >= max_uses
    {
        return Err(AppError::Validation("invite fully used".to_string()));
    }

    // Verify user is a member of the group's community
    let group = group_repo::get_group(&state.db, invite.group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    if !claims.is_owner
        && !community_repo::is_community_member(&state.db, group.community_id, claims.sub).await?
    {
        return Err(AppError::Validation(
            "you must be a member of this community to join this group".to_string(),
        ));
    }

    // Check if already a member
    if group_repo::is_member(&state.db, invite.group_id, claims.sub).await? {
        return Err(AppError::Conflict(
            "already a member of this group".to_string(),
        ));
    }

    // Atomically increment invite usage (checks max_uses in WHERE clause)
    let incremented = invite_repo::increment_usage(&state.db, invite.id).await?;
    if !incremented {
        return Err(AppError::Validation("invite fully used".to_string()));
    }

    // Join the group
    group_repo::join_group(&state.db, invite.group_id, claims.sub).await?;

    Ok(Json(AcceptInviteResponse {
        group_id: group.id,
        group_name: group.name,
    }))
}

// ── Group Assets (Icon / Banner) ──

const MAX_GROUP_ICON_SIZE: usize = 2 * 1024 * 1024; // 2MB
const MAX_GROUP_BANNER_SIZE: usize = 5 * 1024 * 1024; // 5MB
const ALLOWED_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

async fn upload_group_icon(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<GroupResponse>, AppError> {
    let current_group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &current_group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    let (data, ct) = read_image_field(&mut multipart, "icon", MAX_GROUP_ICON_SIZE).await?;
    let ext = image_ext(&ct);

    let asset_dir = std::path::Path::new(&state.config.file_storage_path).join("group_assets");
    tokio::fs::create_dir_all(&asset_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create dir: {e}")))?;

    let filename = format!("{id}_icon.{ext}");
    let file_path = asset_dir.join(&filename);
    write_asset_file(&file_path, &data).await?;

    let icon_url = format!("/api/group-assets/{filename}");
    let group = group_repo::update_group(
        &state.db,
        id,
        None,
        None,
        None,
        None,
        None,
        Some(&icon_url),
        None,
        None,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("group not found".into()))?;

    let count = group_repo::get_member_count(&state.db, id).await?;
    Ok(Json(group_to_response(group, count)))
}

async fn upload_group_banner(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<GroupResponse>, AppError> {
    let current_group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &current_group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    let (data, ct) = read_image_field(&mut multipart, "banner", MAX_GROUP_BANNER_SIZE).await?;
    let ext = image_ext(&ct);

    let asset_dir = std::path::Path::new(&state.config.file_storage_path).join("group_assets");
    tokio::fs::create_dir_all(&asset_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create dir: {e}")))?;

    let filename = format!("{id}_banner.{ext}");
    let file_path = asset_dir.join(&filename);
    write_asset_file(&file_path, &data).await?;

    let banner_url = format!("/api/group-assets/{filename}");
    let group = group_repo::update_group(
        &state.db,
        id,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&banner_url),
        None,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("group not found".into()))?;

    let count = group_repo::get_member_count(&state.db, id).await?;
    Ok(Json(group_to_response(group, count)))
}

const MAX_VOICE_BG_SIZE: usize = 2 * 1024 * 1024; // 2MB

async fn upload_channel_voice_background(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(path): Path<GroupChannelPath>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<ChannelResponse>, AppError> {
    let group = group_repo::get_group(&state.db, path.group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let role = get_effective_group_role(&state.db, &group, claims.sub, claims.is_owner)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    // Verify the channel actually belongs to this group
    let existing = channel_repo::get_channel(&state.db, path.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("channel not found".to_string()))?;
    if existing.group_id != Some(path.group_id) {
        return Err(AppError::Forbidden);
    }

    let (data, ct) = read_image_field(&mut multipart, "background", MAX_VOICE_BG_SIZE).await?;
    let ext = image_ext(&ct);

    let asset_dir = std::path::Path::new(&state.config.file_storage_path).join("group_assets");
    tokio::fs::create_dir_all(&asset_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create dir: {e}")))?;

    let filename = format!("{}_voicebg.{ext}", path.channel_id);
    let file_path = asset_dir.join(&filename);
    write_asset_file(&file_path, &data).await?;

    let bg_url = format!("/api/group-assets/{filename}");
    let channel = channel_repo::update_channel(
        &state.db,
        path.channel_id,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&bg_url),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("channel not found".into()))?;

    Ok(Json(ChannelResponse {
        id: channel.id,
        name: channel.name,
        channel_type: format!("{:?}", channel.channel_type).to_lowercase(),
        topic: channel.topic,
        created_by: channel.created_by,
        created_at: channel.created_at.to_rfc3339(),
        group_id: channel.group_id,
        read_only: channel.read_only,
        slow_mode_seconds: channel.slow_mode_seconds,
        discoverable: channel.discoverable,
        archived: channel.archived,
        voice_background: channel.voice_background,
    }))
}

async fn serve_group_asset(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<([(header::HeaderName, String); 2], Body), AppError> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::Validation("invalid filename".into()));
    }

    let path = std::path::Path::new(&state.config.file_storage_path)
        .join("group_assets")
        .join(&filename);

    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| AppError::NotFound("asset not found".into()))?;

    let content_type = guess_content_type(&filename);
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
        ],
        body,
    ))
}

fn group_to_response(g: Group, member_count: i64) -> GroupResponse {
    GroupResponse {
        id: g.id,
        name: g.name,
        description: g.description,
        icon_url: g.icon_url,
        banner_url: g.banner_url,
        accent_color: g.accent_color,
        owner_id: g.owner_id,
        community_id: g.community_id,
        created_at: g.created_at.to_rfc3339(),
        member_count,
        visibility: g.visibility,
        discoverable: g.discoverable,
        assigned_member_id: g.assigned_member_id,
        allow_invites: g.allow_invites,
    }
}

async fn read_image_field(
    multipart: &mut axum::extract::Multipart,
    field_name: &str,
    max_size: usize,
) -> Result<(Vec<u8>, String), AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart error: {e}")))?
    {
        if field.name() == Some(field_name) {
            content_type = field.content_type().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("read error: {e}")))?;
            if bytes.len() > max_size {
                return Err(AppError::Validation(format!(
                    "file too large (max {}MB)",
                    max_size / (1024 * 1024)
                )));
            }
            file_data = Some(bytes.to_vec());
        }
    }

    let data = file_data.ok_or_else(|| AppError::Validation(format!("no {field_name} field")))?;
    let ct = content_type.ok_or_else(|| AppError::Validation("missing content type".into()))?;

    if !ALLOWED_IMAGE_TYPES.contains(&ct.as_str()) {
        return Err(AppError::Validation(
            "invalid image type (allowed: png, jpeg, webp, gif)".into(),
        ));
    }

    // Validate magic bytes match the declared content type
    let detected = crate::services::file_security::validate_file_type(&data)
        .map_err(|reason| AppError::Validation(format!("invalid file: {reason}")))?;
    if detected != ct {
        return Err(AppError::Validation(
            "file content does not match declared image type".into(),
        ));
    }

    Ok((data, ct))
}

fn image_ext(content_type: &str) -> &'static str {
    match content_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "bin",
    }
}

async fn write_asset_file(path: &std::path::Path, data: &[u8]) -> Result<(), AppError> {
    let mut f = tokio::fs::File::create(path)
        .await
        .map_err(|e| AppError::Internal(format!("create file: {e}")))?;
    f.write_all(data)
        .await
        .map_err(|e| AppError::Internal(format!("write file: {e}")))?;
    f.flush()
        .await
        .map_err(|e| AppError::Internal(format!("flush file: {e}")))?;
    Ok(())
}

fn guess_content_type(filename: &str) -> &'static str {
    if filename.ends_with(".png") {
        "image/png"
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".webp") {
        "image/webp"
    } else if filename.ends_with(".gif") {
        "image/gif"
    } else {
        "application/octet-stream"
    }
}

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    AcceptInviteResponse, ChannelResponse, CreateChannelRequest, CreateGroupRequest,
    CreateInviteRequest, GroupMemberResponse, GroupResponse, InviteInfoResponse, InviteResponse,
    TransferOwnershipRequest, UpdateChannelRequest, UpdateGroupRequest,
};
use chatalot_db::models::channel::ChannelType;
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::repos::{channel_repo, community_repo, group_repo, invite_repo, sender_key_repo};
use rand::Rng as _;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::permissions;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/discover", get(discover_groups))
        .route("/groups/{id}", get(get_group).patch(update_group).delete(delete_group_handler))
        .route(
            "/groups/{id}/transfer-ownership",
            post(transfer_group_ownership),
        )
        .route("/groups/{id}/join", post(join_group))
        .route("/groups/{id}/leave", post(leave_group))
        .route("/groups/{id}/members", get(list_group_members))
        .route("/groups/{id}/channels", get(list_group_channels).post(create_group_channel))
        .route(
            "/groups/{group_id}/channels/{channel_id}",
            patch(update_group_channel).delete(delete_group_channel),
        )
        .route(
            "/groups/{id}/invites",
            get(list_invites).post(create_invite),
        )
        .route("/groups/{id}/invites/{invite_id}", axum::routing::delete(delete_invite))
        .route("/invites/{code}", get(get_invite_info))
        .route("/invites/{code}/accept", post(accept_invite))
}

async fn create_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    if req.name.is_empty() || req.name.len() > 64 {
        return Err(AppError::Validation(
            "group name must be 1-64 characters".to_string(),
        ));
    }

    if let Some(ref desc) = req.description
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

    let effective_role = if claims.is_owner { "instance_admin" } else { &community_role };

    // Look up the community's policy for group creation
    let community = community_repo::get_community(&state.db, req.community_id)
        .await?
        .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    if !permissions::meets_policy(effective_role, &community.who_can_create_groups) {
        return Err(AppError::Forbidden);
    }

    let visibility = req.visibility.as_deref().unwrap_or("public");
    if visibility != "public" && visibility != "private" {
        return Err(AppError::Validation("visibility must be 'public' or 'private'".to_string()));
    }

    let group_id = Uuid::now_v7();
    let group = group_repo::create_group(
        &state.db,
        group_id,
        &req.name,
        req.description.as_deref(),
        claims.sub,
        req.community_id,
        visibility,
    )
    .await?;

    // Auto-create #general text channel
    let channel_id = Uuid::now_v7();
    channel_repo::create_channel(
        &state.db,
        channel_id,
        "general",
        ChannelType::Text,
        None,
        claims.sub,
        Some(group_id),
    )
    .await?;

    // Auto-add all community members to public groups; private groups only have creator
    let mut member_count: i64 = 1;
    if visibility == "public" {
        let member_ids =
            community_repo::list_community_member_user_ids(&state.db, req.community_id)
                .await?;
        for uid in &member_ids {
            if *uid == claims.sub {
                continue; // Creator already added as owner
            }
            if group_repo::join_group(&state.db, group_id, *uid).await.is_ok() {
                member_count += 1;
            }
        }
    }

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        owner_id: group.owner_id,
        community_id: group.community_id,
        created_at: group.created_at.to_rfc3339(),
        member_count,
        visibility: group.visibility,
        discoverable: group.discoverable,
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
                owner_id: g.owner_id,
                community_id: g.community_id,
                created_at: g.created_at.to_rfc3339(),
                member_count: count,
                visibility: g.visibility,
                discoverable: g.discoverable,
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
                owner_id: g.owner_id,
                community_id: g.community_id,
                created_at: g.created_at.to_rfc3339(),
                member_count: count,
                visibility: g.visibility,
                discoverable: g.discoverable,
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
    if !group_repo::is_member(&state.db, id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let group = group_repo::get_group(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let count = group_repo::get_member_count(&state.db, id).await?;

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        owner_id: group.owner_id,
        community_id: group.community_id,
        created_at: group.created_at.to_rfc3339(),
        member_count: count,
        visibility: group.visibility,
        discoverable: group.discoverable,
    }))
}

async fn update_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    let role = group_repo::get_member_role(&state.db, id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    // Validate visibility if provided
    if let Some(ref vis) = req.visibility
        && vis != "public" && vis != "private"
    {
        return Err(AppError::Validation("visibility must be 'public' or 'private'".to_string()));
    }

    if let Some(ref desc) = req.description
        && desc.len() > 2048
    {
        return Err(AppError::Validation(
            "description must be at most 2048 characters".to_string(),
        ));
    }

    let group = group_repo::update_group(
        &state.db,
        id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.visibility.as_deref(),
        req.discoverable,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;

    let count = group_repo::get_member_count(&state.db, id).await?;

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        owner_id: group.owner_id,
        community_id: group.community_id,
        created_at: group.created_at.to_rfc3339(),
        member_count: count,
        visibility: group.visibility,
        discoverable: group.discoverable,
    }))
}

async fn delete_group_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    let role = group_repo::get_member_role(&state.db, id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" {
        return Err(AppError::Forbidden);
    }

    group_repo::delete_group(&state.db, id).await?;
    Ok(())
}

async fn transfer_group_ownership(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(group_id): Path<Uuid>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<(), AppError> {
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
    for ch in &channels {
        let _ = sender_key_repo::delete_distribution(&state.db, ch.id, claims.sub).await;
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
) -> Result<Json<Vec<GroupMemberResponse>>, AppError> {
    if !group_repo::is_member(&state.db, id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let members = group_repo::list_group_members(&state.db, id).await?;
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
    if !group_repo::is_member(&state.db, id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

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
    let role = group_repo::get_member_role(&state.db, group_id, claims.sub)
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
    let members = group_repo::list_group_members(&state.db, group_id).await?;
    for m in members {
        if m.user_id != claims.sub {
            // Creator is already added by create_channel
            let _ = channel_repo::join_channel(&state.db, channel_id, m.user_id).await;
        }
    }

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
    let role = group_repo::get_member_role(&state.db, path.group_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    if let Some(ref topic) = req.topic
        && topic.len() > 512
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

    let channel = channel_repo::update_channel(
        &state.db,
        path.channel_id,
        req.name.as_deref(),
        req.topic.as_deref(),
        req.read_only,
        req.slow_mode_seconds,
        None,
        req.discoverable,
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
    }))
}

async fn delete_group_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(path): Path<GroupChannelPath>,
) -> Result<(), AppError> {
    let role = group_repo::get_member_role(&state.db, path.group_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    channel_repo::delete_channel(&state.db, path.channel_id).await?;
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
    let role = group_repo::get_member_role(&state.db, group_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    let expires_at = req.expires_in_hours.map(|h| {
        chrono::Utc::now() + chrono::Duration::hours(h as i64)
    });

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
    let role = group_repo::get_member_role(&state.db, group_id, claims.sub)
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
    let role = group_repo::get_member_role(&state.db, path.id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }

    invite_repo::delete_invite(&state.db, path.invite_id).await?;
    Ok(())
}

async fn get_invite_info(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<InviteInfoResponse>, AppError> {
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
        return Err(AppError::Conflict("already a member of this group".to_string()));
    }

    // Join the group
    group_repo::join_group(&state.db, invite.group_id, claims.sub).await?;

    // Increment usage
    invite_repo::increment_usage(&state.db, invite.id).await?;

    Ok(Json(AcceptInviteResponse {
        group_id: group.id,
        group_name: group.name,
    }))
}

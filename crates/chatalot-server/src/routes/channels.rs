use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use axum::{Extension, Json, Router};
use chatalot_common::ws_messages::ServerMessage;
use uuid::Uuid;

use chatalot_common::api_types::{
    BanRequest, ChannelMemberResponse, ChannelResponse, CreateChannelRequest,
    TransferOwnershipRequest, UpdateChannelRequest, UpdateRoleRequest,
};
use chatalot_db::models::channel::ChannelType;
use chatalot_db::repos::{channel_repo, sender_key_repo, unread_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::permissions;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/channels", get(list_channels).post(create_channel))
        .route("/channels/{id}", get(get_channel).patch(update_channel))
        .route("/channels/{id}/join", post(join_channel))
        .route("/channels/{id}/leave", post(leave_channel))
        .route("/channels/{id}/members", get(list_channel_members))
        .route(
            "/channels/{id}/members/{user_id}/role",
            patch(update_member_role),
        )
        .route(
            "/channels/{id}/members/{user_id}/kick",
            post(kick_member),
        )
        .route(
            "/channels/{id}/members/{user_id}/ban",
            post(ban_member),
        )
        .route(
            "/channels/{id}/members/{user_id}/unban",
            post(unban_member),
        )
        .route(
            "/channels/{id}/transfer-ownership",
            post(transfer_ownership),
        )
        .route("/channels/unread", get(get_unread_counts))
}

async fn create_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<Json<ChannelResponse>, AppError> {
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

    let id = Uuid::now_v7();
    let channel = channel_repo::create_channel(
        &state.db,
        id,
        &req.name,
        channel_type,
        req.topic.as_deref(),
        claims.sub,
        req.group_id,
    )
    .await?;

    Ok(Json(channel_to_response(&channel)))
}

async fn list_channels(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<ChannelResponse>>, AppError> {
    let channels = channel_repo::list_user_channels(&state.db, claims.sub).await?;
    Ok(Json(channels.iter().map(channel_to_response).collect()))
}

async fn get_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChannelResponse>, AppError> {
    // Verify membership
    if !channel_repo::is_member(&state.db, id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let channel = channel_repo::get_channel(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("channel not found".to_string()))?;

    Ok(Json(channel_to_response(&channel)))
}

async fn update_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> Result<Json<ChannelResponse>, AppError> {
    // Must be channel owner or admin to update
    let role = channel_repo::get_member_role(&state.db, id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_manage_roles(&role) {
        return Err(AppError::Forbidden);
    }

    if let Some(ref name) = req.name
        && (name.is_empty() || name.len() > 64)
    {
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

    if let Some(sms) = req.slow_mode_seconds
        && !(0..=86400).contains(&sms)
    {
        return Err(AppError::Validation(
            "slow_mode_seconds must be between 0 and 86400".to_string(),
        ));
    }

    let channel = channel_repo::update_channel(
        &state.db,
        id,
        req.name.as_deref(),
        req.topic.as_deref(),
        req.read_only,
        req.slow_mode_seconds,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("channel not found".to_string()))?;

    Ok(Json(channel_to_response(&channel)))
}

async fn join_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    // Verify channel exists
    channel_repo::get_channel(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("channel not found".to_string()))?;

    // Check if user is banned
    if channel_repo::is_banned(&state.db, id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    channel_repo::join_channel(&state.db, id, claims.sub).await?;
    Ok(())
}

async fn leave_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    channel_repo::leave_channel(&state.db, id, claims.sub).await?;
    Ok(())
}

async fn list_channel_members(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ChannelMemberResponse>>, AppError> {
    if !channel_repo::is_member(&state.db, id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let members = channel_repo::list_members_with_users(&state.db, id).await?;
    let response: Vec<ChannelMemberResponse> = members
        .into_iter()
        .map(|m| ChannelMemberResponse {
            user_id: m.user_id,
            username: m.username,
            display_name: m.display_name,
            avatar_url: m.avatar_url,
            role: m.role,
            joined_at: m.joined_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

async fn update_member_role(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<(), AppError> {
    let actor_role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_manage_roles(&actor_role) {
        return Err(AppError::Forbidden);
    }

    if target_user_id == claims.sub {
        return Err(AppError::Validation(
            "cannot change your own role".to_string(),
        ));
    }

    if req.role != "admin" && req.role != "member" {
        return Err(AppError::Validation(
            "role must be 'admin' or 'member'".to_string(),
        ));
    }

    if !channel_repo::update_member_role(&state.db, channel_id, target_user_id, &req.role).await? {
        return Err(AppError::NotFound("member not found".to_string()));
    }

    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::MemberRoleUpdated {
            channel_id,
            user_id: target_user_id,
            role: req.role,
        },
    );

    Ok(())
}

async fn kick_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    let actor_role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    let target_role = channel_repo::get_member_role(&state.db, channel_id, target_user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("member not found".to_string()))?;

    if !permissions::can_moderate(&actor_role, &target_role) {
        return Err(AppError::Forbidden);
    }

    channel_repo::leave_channel(&state.db, channel_id, target_user_id).await?;

    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::MemberKicked {
            channel_id,
            user_id: target_user_id,
            kicked_by: claims.sub,
        },
    );

    // Delete kicked user's sender key and trigger rotation for remaining members
    let _ = sender_key_repo::delete_distribution(&state.db, channel_id, target_user_id).await;
    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::SenderKeyRotationRequired {
            channel_id,
            reason: "member_removed".to_string(),
        },
    );

    Ok(())
}

async fn ban_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<BanRequest>,
) -> Result<(), AppError> {
    let actor_role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    let target_role = channel_repo::get_member_role(&state.db, channel_id, target_user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("member not found".to_string()))?;

    if !permissions::can_moderate(&actor_role, &target_role) {
        return Err(AppError::Forbidden);
    }

    channel_repo::ban_user(
        &state.db,
        channel_id,
        target_user_id,
        claims.sub,
        req.reason.as_deref(),
    )
    .await?;

    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::MemberBanned {
            channel_id,
            user_id: target_user_id,
            banned_by: claims.sub,
        },
    );

    // Delete banned user's sender key and trigger rotation for remaining members
    let _ = sender_key_repo::delete_distribution(&state.db, channel_id, target_user_id).await;
    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::SenderKeyRotationRequired {
            channel_id,
            reason: "member_removed".to_string(),
        },
    );

    Ok(())
}

async fn unban_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    let actor_role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    // Admins and owners can unban
    if !permissions::can_delete_others_messages(&actor_role) {
        return Err(AppError::Forbidden);
    }

    if !channel_repo::unban_user(&state.db, channel_id, target_user_id).await? {
        return Err(AppError::NotFound("ban not found".to_string()));
    }

    Ok(())
}

async fn transfer_ownership(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<(), AppError> {
    // Only the current owner can transfer
    let actor_role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if actor_role != "owner" {
        return Err(AppError::Forbidden);
    }

    // Target must be a member
    if !channel_repo::is_member(&state.db, channel_id, req.new_owner_id).await? {
        return Err(AppError::Validation(
            "target user must be a member of this channel".to_string(),
        ));
    }

    channel_repo::transfer_ownership(&state.db, channel_id, claims.sub, req.new_owner_id).await?;

    // Broadcast role changes for both users
    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::MemberRoleUpdated {
            channel_id,
            user_id: claims.sub,
            role: "admin".to_string(),
        },
    );
    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::MemberRoleUpdated {
            channel_id,
            user_id: req.new_owner_id,
            role: "owner".to_string(),
        },
    );

    Ok(())
}

async fn get_unread_counts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let counts = unread_repo::get_all_unread_counts(&state.db, claims.sub).await?;
    let result: Vec<serde_json::Value> = counts
        .iter()
        .map(|c| {
            serde_json::json!({
                "channel_id": c.channel_id,
                "unread_count": c.unread_count
            })
        })
        .collect();
    Ok(Json(result))
}

fn channel_to_response(ch: &chatalot_db::models::channel::Channel) -> ChannelResponse {
    ChannelResponse {
        id: ch.id,
        name: ch.name.clone(),
        channel_type: format!("{:?}", ch.channel_type).to_lowercase(),
        topic: ch.topic.clone(),
        created_by: ch.created_by,
        created_at: ch.created_at.to_rfc3339(),
        group_id: ch.group_id,
        read_only: ch.read_only,
        slow_mode_seconds: ch.slow_mode_seconds,
    }
}

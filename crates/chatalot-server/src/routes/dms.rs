use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{ChannelResponse, CreateDmRequest, DmChannelResponse, UserPublic};
use chatalot_db::repos::{block_repo, community_repo, dm_repo};
use chatalot_db::models::user::User;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dms", get(list_dms).post(create_dm))
}

async fn create_dm(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateDmRequest>,
) -> Result<Json<DmChannelResponse>, AppError> {
    if req.target_user_id == claims.sub {
        return Err(AppError::Validation("cannot DM yourself".to_string()));
    }

    // Verify target user exists
    let target = chatalot_db::repos::user_repo::find_by_id(&state.db, req.target_user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    // Check if either user has blocked the other
    if block_repo::is_blocked_either_way(&state.db, claims.sub, target.id).await? {
        return Err(AppError::Validation("cannot create DM with this user".to_string()));
    }

    // DMs require shared community membership (instance owner bypasses for moderation)
    if !claims.is_owner
        && !community_repo::shares_community(&state.db, claims.sub, target.id).await?
    {
        return Err(AppError::Forbidden);
    }

    let channel_id = Uuid::now_v7();
    let channel = dm_repo::get_or_create_dm(
        &state.db,
        channel_id,
        claims.sub,
        target.id,
    )
    .await?;

    // Don't notify the target user yet — they'll be notified when the
    // first message is actually sent (see ws/handler.rs).

    Ok(Json(DmChannelResponse {
        channel: ChannelResponse {
            id: channel.id,
            name: channel.name.clone(),
            channel_type: "dm".to_string(),
            topic: channel.topic.clone(),
            created_by: channel.created_by,
            created_at: channel.created_at.to_rfc3339(),
            group_id: None,
            read_only: false,
            slow_mode_seconds: 0,
            discoverable: true,
            archived: false,
            voice_background: None,
        },
        other_user: user_to_public(&target),
    }))
}

async fn list_dms(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<DmChannelResponse>>, AppError> {
    let dms = dm_repo::list_user_dms(&state.db, claims.sub).await?;

    let responses = dms
        .into_iter()
        .map(|(channel, other_user)| DmChannelResponse {
            channel: ChannelResponse {
                id: channel.id,
                name: channel.name.clone(),
                channel_type: "dm".to_string(),
                topic: channel.topic.clone(),
                created_by: channel.created_by,
                created_at: channel.created_at.to_rfc3339(),
                group_id: None,
                read_only: false,
                slow_mode_seconds: 0,
                discoverable: true,
                archived: false,
                voice_background: None,
            },
            other_user: user_to_public(&other_user),
        })
        .collect();

    Ok(Json(responses))
}

fn user_to_public(u: &User) -> UserPublic {
    UserPublic {
        id: u.id,
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        avatar_url: u.avatar_url.clone(),
        banner_url: u.banner_url.clone(),
        status: u.status.clone(),
        custom_status: u.custom_status.clone(),
        is_admin: u.is_admin,
        is_owner: u.is_owner,
        created_at: Some(u.created_at.to_rfc3339()),
    }
}

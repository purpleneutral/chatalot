use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    AcceptCommunityInviteResponse, CommunityBanRequest, CommunityInviteInfoResponse,
    CommunityInviteResponse, CommunityMemberResponse, CommunityResponse,
    CreateCommunityInviteRequest, CreateCommunityRequest, SetCommunityRoleRequest,
    SetNicknameRequest, TransferCommunityOwnershipRequest, UpdateCommunityRequest,
};
use chatalot_db::repos::community_repo;
use rand::Rng as _;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::middleware::community_gate::CommunityContext;

/// Public routes (no community gate — user may not be member).
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/communities", get(list_my_communities).post(create_community))
        .route("/community-invites/{code}", get(get_community_invite_info))
        .route(
            "/community-invites/{code}/accept",
            post(accept_community_invite),
        )
}

/// Community-scoped routes (behind community_gate middleware).
pub fn gated_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/communities/{cid}",
            get(get_community).patch(update_community).delete(delete_community),
        )
        .route(
            "/communities/{cid}/transfer-ownership",
            post(transfer_ownership),
        )
        .route("/communities/{cid}/leave", post(leave_community))
        .route("/communities/{cid}/members", get(list_members))
        .route(
            "/communities/{cid}/members/{uid}/role",
            put(set_member_role),
        )
        .route(
            "/communities/{cid}/members/{uid}/nickname",
            put(set_nickname),
        )
        .route(
            "/communities/{cid}/members/{uid}",
            delete(kick_member),
        )
        .route("/communities/{cid}/bans", get(list_bans))
        .route(
            "/communities/{cid}/bans/{uid}",
            post(ban_member).delete(unban_member),
        )
        .route(
            "/communities/{cid}/invites",
            get(list_invites).post(create_invite),
        )
        .route(
            "/communities/{cid}/invites/{iid}",
            delete(delete_invite),
        )
        .route(
            "/communities/{cid}/groups",
            get(list_community_groups),
        )
}

// ── Public Handlers ──

async fn create_community(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateCommunityRequest>,
) -> Result<Json<CommunityResponse>, AppError> {
    // Check creation mode
    if state.config.community_creation_mode == "admin_only" && !claims.is_admin {
        return Err(AppError::Forbidden);
    }

    if req.name.is_empty() || req.name.len() > 64 {
        return Err(AppError::Validation(
            "community name must be 1-64 characters".to_string(),
        ));
    }

    let id = Uuid::now_v7();
    let community = community_repo::create_community(
        &state.db,
        id,
        &req.name,
        req.description.as_deref(),
        None,
        claims.sub,
    )
    .await?;

    Ok(Json(CommunityResponse {
        id: community.id,
        name: community.name,
        description: community.description,
        icon_url: community.icon_url,
        owner_id: community.owner_id,
        created_at: community.created_at.to_rfc3339(),
        member_count: 1,
    }))
}

async fn list_my_communities(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<CommunityResponse>>, AppError> {
    let communities = community_repo::list_user_communities(&state.db, claims.sub).await?;
    let mut responses = Vec::with_capacity(communities.len());
    for c in communities {
        let count = community_repo::get_community_member_count(&state.db, c.id).await?;
        responses.push(CommunityResponse {
            id: c.id,
            name: c.name,
            description: c.description,
            icon_url: c.icon_url,
            owner_id: c.owner_id,
            created_at: c.created_at.to_rfc3339(),
            member_count: count,
        });
    }
    Ok(Json(responses))
}

async fn get_community_invite_info(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<CommunityInviteInfoResponse>, AppError> {
    let invite = community_repo::get_community_invite_by_code(&state.db, &code)
        .await?
        .ok_or_else(|| AppError::NotFound("invite not found".to_string()))?;

    if let Some(expires_at) = invite.expires_at
        && expires_at < chrono::Utc::now()
    {
        return Err(AppError::NotFound("invite expired".to_string()));
    }

    if let Some(max_uses) = invite.max_uses
        && invite.used_count >= max_uses
    {
        return Err(AppError::NotFound("invite fully used".to_string()));
    }

    let community = community_repo::get_community(&state.db, invite.community_id)
        .await?
        .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    let count = community_repo::get_community_member_count(&state.db, community.id).await?;

    Ok(Json(CommunityInviteInfoResponse {
        community_name: community.name,
        community_description: community.description,
        member_count: count,
        code: invite.code,
    }))
}

async fn accept_community_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(code): Path<String>,
) -> Result<Json<AcceptCommunityInviteResponse>, AppError> {
    let invite = community_repo::get_community_invite_by_code(&state.db, &code)
        .await?
        .ok_or_else(|| AppError::NotFound("invite not found".to_string()))?;

    if let Some(expires_at) = invite.expires_at
        && expires_at < chrono::Utc::now()
    {
        return Err(AppError::Validation("invite expired".to_string()));
    }

    if let Some(max_uses) = invite.max_uses
        && invite.used_count >= max_uses
    {
        return Err(AppError::Validation("invite fully used".to_string()));
    }

    // Check if banned
    if community_repo::is_banned_from_community(&state.db, invite.community_id, claims.sub)
        .await?
    {
        return Err(AppError::Forbidden);
    }

    // Check if already a member
    if community_repo::is_community_member(&state.db, invite.community_id, claims.sub).await? {
        return Err(AppError::Conflict(
            "already a member of this community".to_string(),
        ));
    }

    community_repo::join_community(&state.db, invite.community_id, claims.sub).await?;
    community_repo::increment_community_invite_usage(&state.db, invite.id).await?;

    let community = community_repo::get_community(&state.db, invite.community_id)
        .await?
        .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    Ok(Json(AcceptCommunityInviteResponse {
        community_id: community.id,
        community_name: community.name,
    }))
}

// ── Gated Handlers (community membership pre-validated) ──

async fn get_community(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<Json<CommunityResponse>, AppError> {
    let community = community_repo::get_community(&state.db, ctx.community_id)
        .await?
        .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    let count = community_repo::get_community_member_count(&state.db, ctx.community_id).await?;

    Ok(Json(CommunityResponse {
        id: community.id,
        name: community.name,
        description: community.description,
        icon_url: community.icon_url,
        owner_id: community.owner_id,
        created_at: community.created_at.to_rfc3339(),
        member_count: count,
    }))
}

async fn update_community(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
    Json(req): Json<UpdateCommunityRequest>,
) -> Result<Json<CommunityResponse>, AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let community = community_repo::update_community(
        &state.db,
        ctx.community_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.icon_url.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    let count = community_repo::get_community_member_count(&state.db, ctx.community_id).await?;

    Ok(Json(CommunityResponse {
        id: community.id,
        name: community.name,
        description: community.description,
        icon_url: community.icon_url,
        owner_id: community.owner_id,
        created_at: community.created_at.to_rfc3339(),
        member_count: count,
    }))
}

async fn delete_community(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<(), AppError> {
    if !ctx.is_owner() {
        return Err(AppError::Forbidden);
    }

    community_repo::delete_community(&state.db, ctx.community_id).await?;
    Ok(())
}

async fn transfer_ownership(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Json(req): Json<TransferCommunityOwnershipRequest>,
) -> Result<(), AppError> {
    if !ctx.is_owner() {
        return Err(AppError::Forbidden);
    }

    if !community_repo::is_community_member(&state.db, ctx.community_id, req.new_owner_id).await? {
        return Err(AppError::Validation(
            "target user must be a member of this community".to_string(),
        ));
    }

    community_repo::transfer_community_ownership(
        &state.db,
        ctx.community_id,
        claims.sub,
        req.new_owner_id,
    )
    .await?;

    Ok(())
}

async fn leave_community(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<(), AppError> {
    if ctx.role == "owner" {
        return Err(AppError::Validation(
            "owner cannot leave; transfer ownership or delete the community instead".to_string(),
        ));
    }

    community_repo::leave_community(&state.db, ctx.community_id, claims.sub).await?;
    Ok(())
}

async fn list_members(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<Json<Vec<CommunityMemberResponse>>, AppError> {
    let members = community_repo::list_community_members(&state.db, ctx.community_id).await?;
    Ok(Json(
        members
            .into_iter()
            .map(|m| CommunityMemberResponse {
                user_id: m.user_id,
                username: m.username,
                display_name: m.display_name,
                avatar_url: m.avatar_url,
                role: m.role,
                nickname: m.nickname,
                joined_at: m.joined_at.to_rfc3339(),
            })
            .collect(),
    ))
}

#[derive(serde::Deserialize)]
struct CommunityMemberPath {
    #[allow(dead_code)]
    cid: Uuid,
    uid: Uuid,
}

async fn set_member_role(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityMemberPath>,
    Json(req): Json<SetCommunityRoleRequest>,
) -> Result<(), AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    if path.uid == claims.sub {
        return Err(AppError::Validation(
            "you cannot change your own role".to_string(),
        ));
    }

    // Validate role
    if !matches!(req.role.as_str(), "member" | "moderator" | "admin") {
        return Err(AppError::Validation(
            "role must be member, moderator, or admin".to_string(),
        ));
    }

    // Cannot change the role of the owner
    let target_role =
        community_repo::get_community_member_role(&state.db, ctx.community_id, path.uid)
            .await?
            .ok_or_else(|| AppError::NotFound("member not found".to_string()))?;

    if target_role == "owner" {
        return Err(AppError::Validation(
            "cannot change the owner's role; use transfer-ownership instead".to_string(),
        ));
    }

    // Only owner/instance_admin can promote to admin
    if req.role == "admin" && !ctx.is_owner() {
        return Err(AppError::Forbidden);
    }

    community_repo::set_community_member_role(&state.db, ctx.community_id, path.uid, &req.role)
        .await?;

    Ok(())
}

async fn set_nickname(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityMemberPath>,
    Json(req): Json<SetNicknameRequest>,
) -> Result<(), AppError> {
    // Users can set their own nickname; admins can set anyone's
    if path.uid != claims.sub && !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    if let Some(ref nick) = req.nickname
        && (nick.is_empty() || nick.len() > 64)
    {
        return Err(AppError::Validation(
            "nickname must be 1-64 characters".to_string(),
        ));
    }

    community_repo::set_community_nickname(
        &state.db,
        ctx.community_id,
        path.uid,
        req.nickname.as_deref(),
    )
    .await?;

    Ok(())
}

async fn kick_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityMemberPath>,
) -> Result<(), AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }

    if path.uid == claims.sub {
        return Err(AppError::Validation(
            "you cannot kick yourself; use leave instead".to_string(),
        ));
    }

    // Cannot kick the owner
    let target_role =
        community_repo::get_community_member_role(&state.db, ctx.community_id, path.uid)
            .await?
            .ok_or_else(|| AppError::NotFound("member not found".to_string()))?;

    if target_role == "owner" {
        return Err(AppError::Validation("cannot kick the owner".to_string()));
    }

    // Moderators can only kick members, not admins
    if ctx.role == "moderator" && target_role != "member" {
        return Err(AppError::Forbidden);
    }

    community_repo::leave_community(&state.db, ctx.community_id, path.uid).await?;
    Ok(())
}

// ── Bans ──

async fn list_bans(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<Json<Vec<chatalot_db::models::community::CommunityBanInfo>>, AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }

    let bans = community_repo::list_community_bans(&state.db, ctx.community_id).await?;
    Ok(Json(bans))
}

#[derive(serde::Deserialize)]
struct CommunityBanPath {
    #[allow(dead_code)]
    cid: Uuid,
    uid: Uuid,
}

async fn ban_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityBanPath>,
    Json(req): Json<CommunityBanRequest>,
) -> Result<(), AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }

    if path.uid == claims.sub {
        return Err(AppError::Validation(
            "you cannot ban yourself".to_string(),
        ));
    }

    // Cannot ban the owner
    if let Some(target_role) =
        community_repo::get_community_member_role(&state.db, ctx.community_id, path.uid).await?
    {
        if target_role == "owner" {
            return Err(AppError::Validation("cannot ban the owner".to_string()));
        }
        // Moderators can only ban members
        if ctx.role == "moderator" && target_role != "member" {
            return Err(AppError::Forbidden);
        }
    }

    community_repo::ban_from_community(
        &state.db,
        ctx.community_id,
        path.uid,
        claims.sub,
        req.reason.as_deref(),
    )
    .await?;

    Ok(())
}

async fn unban_member(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityBanPath>,
) -> Result<(), AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }

    let removed = community_repo::unban_from_community(&state.db, ctx.community_id, path.uid).await?;
    if !removed {
        return Err(AppError::NotFound("ban not found".to_string()));
    }

    Ok(())
}

// ── Invites ──

async fn list_invites(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<Json<Vec<CommunityInviteResponse>>, AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let invites = community_repo::list_community_invites(&state.db, ctx.community_id).await?;
    Ok(Json(
        invites
            .into_iter()
            .map(|i| CommunityInviteResponse {
                id: i.id,
                code: i.code,
                community_id: i.community_id,
                max_uses: i.max_uses,
                used_count: i.used_count,
                expires_at: i.expires_at.map(|t| t.to_rfc3339()),
                created_at: i.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn create_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Json(req): Json<CreateCommunityInviteRequest>,
) -> Result<Json<CommunityInviteResponse>, AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let expires_at = req.expires_in_hours.map(|h| {
        chrono::Utc::now()
            + chrono::Duration::try_hours(h as i64)
                .unwrap_or(chrono::Duration::try_hours(24).unwrap())
    });

    let id = Uuid::now_v7();
    let code = generate_invite_code();

    let invite = community_repo::create_community_invite(
        &state.db,
        id,
        ctx.community_id,
        &code,
        claims.sub,
        req.max_uses,
        expires_at,
    )
    .await?;

    Ok(Json(CommunityInviteResponse {
        id: invite.id,
        code: invite.code,
        community_id: invite.community_id,
        max_uses: invite.max_uses,
        used_count: invite.used_count,
        expires_at: invite.expires_at.map(|t| t.to_rfc3339()),
        created_at: invite.created_at.to_rfc3339(),
    }))
}

#[derive(serde::Deserialize)]
struct CommunityInvitePath {
    #[allow(dead_code)]
    cid: Uuid,
    iid: Uuid,
}

async fn delete_invite(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityInvitePath>,
) -> Result<(), AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let deleted = community_repo::delete_community_invite(&state.db, path.iid).await?;
    if !deleted {
        return Err(AppError::NotFound("invite not found".to_string()));
    }

    Ok(())
}

// ── Groups (community-scoped) ──

async fn list_community_groups(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<Json<Vec<chatalot_common::api_types::GroupResponse>>, AppError> {
    let groups = chatalot_db::repos::group_repo::list_community_groups(&state.db, ctx.community_id)
        .await?;

    let mut responses = Vec::with_capacity(groups.len());
    for g in groups {
        let count = chatalot_db::repos::group_repo::get_member_count(&state.db, g.id).await?;
        responses.push(chatalot_common::api_types::GroupResponse {
            id: g.id,
            name: g.name,
            description: g.description,
            owner_id: g.owner_id,
            community_id: g.community_id,
            created_at: g.created_at.to_rfc3339(),
            member_count: count,
        });
    }
    Ok(Json(responses))
}

// ── Helpers ──

fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

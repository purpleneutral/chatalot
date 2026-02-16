use std::sync::{Arc, LazyLock};

static COLOR_HEX_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^#[0-9a-fA-F]{3,8}$").unwrap());

use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use axum::body::Body;
use axum::http::header;
use tokio::io::AsyncWriteExt;

use chatalot_common::api_types::{
    AcceptCommunityInviteResponse, CommunityBanRequest, CommunityInviteInfoResponse,
    CommunityInviteResponse, CommunityMemberResponse, CommunityResponse,
    CreateCommunityInviteRequest, CreateCommunityRequest, CreateTimeoutRequest,
    CreateWarningRequest, CustomEmojiResponse, PaginationQuery, SetCommunityRoleRequest,
    SetNicknameRequest, TimeoutResponse, TransferCommunityOwnershipRequest, UpdateCommunityRequest,
    WarningResponse,
};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::repos::{
    channel_repo, community_repo, custom_emoji_repo, timeout_repo, user_repo, warning_repo,
};
use rand::Rng as _;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::middleware::community_gate::CommunityContext;
use crate::services::css_sanitizer;

/// Public routes (no community gate — user may not be member).
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/communities",
            get(list_my_communities).post(create_community),
        )
        .route("/community-invites/{code}", get(get_community_invite_info))
        .route(
            "/community-invites/{code}/accept",
            post(accept_community_invite),
        )
        .route("/emojis/{id}", get(serve_emoji))
        .route("/community-assets/{filename}", get(serve_community_asset))
}

/// Community-scoped routes (behind community_gate middleware).
pub fn gated_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/communities/{cid}",
            get(get_community)
                .patch(update_community)
                .delete(delete_community),
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
        .route("/communities/{cid}/members/{uid}", delete(kick_member))
        .route("/communities/{cid}/bans", get(list_bans))
        .route(
            "/communities/{cid}/bans/{uid}",
            post(ban_member).delete(unban_member),
        )
        .route(
            "/communities/{cid}/invites",
            get(list_invites).post(create_invite),
        )
        .route("/communities/{cid}/invites/{iid}", delete(delete_invite))
        .route("/communities/{cid}/groups", get(list_community_groups))
        // Timeout & warn
        .route(
            "/communities/{cid}/channels/{chid}/timeout",
            post(create_timeout),
        )
        .route(
            "/communities/{cid}/channels/{chid}/timeout/{uid}",
            delete(remove_timeout),
        )
        .route(
            "/communities/{cid}/channels/{chid}/warn",
            post(create_warning),
        )
        .route(
            "/communities/{cid}/channels/{chid}/warnings/{uid}",
            get(list_warnings),
        )
        // Community assets
        .route("/communities/{cid}/icon", post(upload_community_icon))
        .route("/communities/{cid}/banner", post(upload_community_banner))
        // Custom emoji
        .route(
            "/communities/{cid}/emojis",
            get(list_emojis).post(upload_emoji),
        )
        .route("/communities/{cid}/emojis/{eid}", delete(delete_emoji))
}

// ── Public Handlers ──

async fn create_community(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateCommunityRequest>,
) -> Result<Json<CommunityResponse>, AppError> {
    // Check creation mode
    if state.config.community_creation_mode == "admin_only" && !claims.is_admin && !claims.is_owner
    {
        return Err(AppError::Forbidden);
    }

    let name = req.name.trim();
    if name.is_empty() || name.len() > 64 {
        return Err(AppError::Validation(
            "community name must be 1-64 characters".to_string(),
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

    let id = Uuid::now_v7();
    let community =
        community_repo::create_community(&state.db, id, name, description, None, claims.sub)
            .await?;

    Ok(Json(CommunityResponse {
        id: community.id,
        name: community.name,
        description: community.description,
        icon_url: community.icon_url,
        banner_url: community.banner_url,
        owner_id: community.owner_id,
        created_at: community.created_at.to_rfc3339(),
        member_count: 1,
        who_can_create_groups: community.who_can_create_groups,
        who_can_create_invites: community.who_can_create_invites,
        discoverable: community.discoverable,
        community_theme: community.community_theme,
        welcome_message: community.welcome_message,
    }))
}

async fn list_my_communities(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<CommunityResponse>>, AppError> {
    let communities = community_repo::list_user_communities(&state.db, claims.sub).await?;
    let community_ids: Vec<Uuid> = communities.iter().map(|c| c.id).collect();
    let counts = community_repo::get_community_member_counts(&state.db, &community_ids).await?;
    let responses = communities
        .into_iter()
        .map(|c| {
            let count = counts.get(&c.id).copied().unwrap_or(0);
            CommunityResponse {
                id: c.id,
                name: c.name,
                description: c.description,
                icon_url: c.icon_url,
                banner_url: c.banner_url,
                owner_id: c.owner_id,
                created_at: c.created_at.to_rfc3339(),
                member_count: count,
                who_can_create_groups: c.who_can_create_groups,
                who_can_create_invites: c.who_can_create_invites,
                discoverable: c.discoverable,
                community_theme: c.community_theme,
                welcome_message: c.welcome_message,
            }
        })
        .collect();
    Ok(Json(responses))
}

async fn get_community_invite_info(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<CommunityInviteInfoResponse>, AppError> {
    if code.len() > 100 {
        return Err(AppError::Validation("invalid invite code".to_string()));
    }

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

    let community = community_repo::get_community(&state.db, invite.community_id)
        .await?
        .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    let count = community_repo::get_community_member_count(&state.db, community.id).await?;

    // Redact info for non-discoverable communities
    let (display_name, display_desc) = if community.discoverable {
        (community.name, community.description)
    } else {
        ("Private Community".to_string(), None)
    };

    Ok(Json(CommunityInviteInfoResponse {
        community_name: display_name,
        community_description: display_desc,
        member_count: count,
        code: invite.code,
    }))
}

async fn accept_community_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(code): Path<String>,
) -> Result<Json<AcceptCommunityInviteResponse>, AppError> {
    if code.len() > 100 {
        return Err(AppError::Validation("invalid invite code".to_string()));
    }

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
    if community_repo::is_banned_from_community(&state.db, invite.community_id, claims.sub).await? {
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

    // Groups are isolated — new members must be explicitly invited to groups

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
        banner_url: community.banner_url,
        owner_id: community.owner_id,
        created_at: community.created_at.to_rfc3339(),
        member_count: count,
        who_can_create_groups: community.who_can_create_groups,
        who_can_create_invites: community.who_can_create_invites,
        discoverable: community.discoverable,
        community_theme: community.community_theme,
        welcome_message: community.welcome_message,
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

    let name = req.name.as_deref().map(str::trim);
    if let Some(n) = name
        && (n.is_empty() || n.len() > 64)
    {
        return Err(AppError::Validation(
            "community name must be 1-64 characters".to_string(),
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

    // Validate policy fields if provided
    let valid_policies = ["everyone", "moderator", "admin"];
    if let Some(ref p) = req.who_can_create_groups
        && !valid_policies.contains(&p.as_str())
    {
        return Err(AppError::Validation(
            "who_can_create_groups must be 'everyone', 'moderator', or 'admin'".to_string(),
        ));
    }
    if let Some(ref p) = req.who_can_create_invites
        && !valid_policies.contains(&p.as_str())
    {
        return Err(AppError::Validation(
            "who_can_create_invites must be 'everyone', 'moderator', or 'admin'".to_string(),
        ));
    }

    // Validate welcome_message length
    if let Some(ref msg) = req.welcome_message
        && msg.len() > 2000
    {
        return Err(AppError::Validation(
            "welcome_message must be at most 2000 characters".to_string(),
        ));
    }

    // Validate community_theme if provided
    let validated_theme = if let Some(ref theme) = req.community_theme {
        let raw = serde_json::to_string(theme)
            .map_err(|e| AppError::Validation(format!("invalid theme JSON: {e}")))?;
        if raw.len() > 8192 {
            return Err(AppError::Validation(
                "community_theme too large (max 8KB)".into(),
            ));
        }
        let obj = theme
            .as_object()
            .ok_or_else(|| AppError::Validation("community_theme must be a JSON object".into()))?;

        const ALLOWED_THEME_KEYS: &[&str] = &[
            "accent",
            "accentHover",
            "bgPrimary",
            "bgSecondary",
            "bgTertiary",
            "textPrimary",
            "textSecondary",
            "customCss",
        ];
        let mut sanitized = serde_json::Map::new();
        for (key, value) in obj {
            if !ALLOWED_THEME_KEYS.contains(&key.as_str()) {
                return Err(AppError::Validation(format!("unknown theme key: {key}")));
            }
            if key == "customCss" {
                let css = value
                    .as_str()
                    .ok_or_else(|| AppError::Validation("customCss must be a string".into()))?;
                let clean = css_sanitizer::sanitize_css(css)
                    .map_err(|e| AppError::Validation(format!("customCss: {e}")))?;
                sanitized.insert(key.clone(), serde_json::Value::String(clean));
            } else {
                let color = value
                    .as_str()
                    .ok_or_else(|| AppError::Validation(format!("{key} must be a string")))?;
                if !COLOR_HEX_RE.is_match(color) {
                    return Err(AppError::Validation(format!(
                        "{key} must be a hex color (e.g. #ff0000)"
                    )));
                }
                sanitized.insert(key.clone(), value.clone());
            }
        }
        Some(serde_json::Value::Object(sanitized))
    } else {
        None
    };

    let community = community_repo::update_community(
        &state.db,
        ctx.community_id,
        name,
        description,
        req.icon_url.as_deref(),
        req.who_can_create_groups.as_deref(),
        req.who_can_create_invites.as_deref(),
        req.discoverable,
        req.banner_url.as_deref(),
        validated_theme.as_ref(),
        req.welcome_message.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    let count = community_repo::get_community_member_count(&state.db, ctx.community_id).await?;

    // Broadcast community settings change to all connected users
    state.connections.broadcast_all(ServerMessage::CommunityUpdated {
        community_id: community.id,
        name: community.name.clone(),
        description: community.description.clone(),
        icon_url: community.icon_url.clone(),
        banner_url: community.banner_url.clone(),
        community_theme: community.community_theme.clone(),
        welcome_message: community.welcome_message.clone(),
    });

    Ok(Json(CommunityResponse {
        id: community.id,
        name: community.name,
        description: community.description,
        icon_url: community.icon_url,
        banner_url: community.banner_url,
        owner_id: community.owner_id,
        created_at: community.created_at.to_rfc3339(),
        member_count: count,
        who_can_create_groups: community.who_can_create_groups,
        who_can_create_invites: community.who_can_create_invites,
        discoverable: community.discoverable,
        community_theme: community.community_theme,
        welcome_message: community.welcome_message,
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
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<CommunityMemberResponse>>, AppError> {
    let limit = pagination.limit.unwrap_or(200).clamp(1, 500);
    let offset = pagination.offset.unwrap_or(0).max(0);
    let members =
        community_repo::list_community_members(&state.db, ctx.community_id, limit, offset).await?;
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

    // Empty string clears the nickname; None means no change
    let nickname = req
        .nickname
        .as_deref()
        .map(str::trim)
        .and_then(|n| if n.is_empty() { None } else { Some(n) });
    if let Some(nick) = nickname
        && nick.len() > 64
    {
        return Err(AppError::Validation(
            "nickname must be 1-64 characters".to_string(),
        ));
    }

    community_repo::set_community_nickname(&state.db, ctx.community_id, path.uid, nickname).await?;

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

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "community_kick",
        None,
        None,
        Some(serde_json::json!({
            "community_id": ctx.community_id,
            "target_user_id": path.uid,
        })),
    )
    .await?;

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
        return Err(AppError::Validation("you cannot ban yourself".to_string()));
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

    let reason = req
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    if let Some(r) = reason
        && r.len() > 500
    {
        return Err(AppError::Validation(
            "reason must be at most 500 characters".to_string(),
        ));
    }
    community_repo::ban_from_community(&state.db, ctx.community_id, path.uid, claims.sub, reason)
        .await?;

    Ok(())
}

async fn unban_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityBanPath>,
) -> Result<(), AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }

    let removed =
        community_repo::unban_from_community(&state.db, ctx.community_id, path.uid).await?;
    if !removed {
        return Err(AppError::NotFound("ban not found".to_string()));
    }

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "community_unban",
        None,
        None,
        Some(serde_json::json!({
            "community_id": ctx.community_id,
            "target_user_id": path.uid,
        })),
    )
    .await?;

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
    // Check community policy for who can create invites
    let community = community_repo::get_community(&state.db, ctx.community_id)
        .await?
        .ok_or_else(|| AppError::NotFound("community not found".to_string()))?;

    if !crate::permissions::meets_policy(&ctx.role, &community.who_can_create_invites) {
        return Err(AppError::Forbidden);
    }

    if let Some(h) = req.expires_in_hours
        && !(1..=8760).contains(&h)
    {
        return Err(AppError::Validation(
            "expires_in_hours must be between 1 and 8760".into(),
        ));
    }
    let expires_at = req.expires_in_hours.map(|h| {
        chrono::Utc::now()
            + chrono::Duration::try_hours(h as i64)
                .unwrap_or(chrono::TimeDelta::hours(24))
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

    let deleted =
        community_repo::delete_community_invite(&state.db, path.iid, ctx.community_id).await?;
    if !deleted {
        return Err(AppError::NotFound("invite not found".to_string()));
    }

    Ok(())
}

// ── Groups (community-scoped) ──

async fn list_community_groups(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<Json<Vec<chatalot_common::api_types::GroupResponse>>, AppError> {
    let groups = chatalot_db::repos::group_repo::list_community_groups(
        &state.db,
        ctx.community_id,
        claims.sub,
    )
    .await?;

    let group_ids: Vec<Uuid> = groups.iter().map(|g| g.id).collect();
    let counts = chatalot_db::repos::group_repo::get_member_counts(&state.db, &group_ids).await?;
    let responses = groups
        .into_iter()
        .map(|g| {
            let count = counts.get(&g.id).copied().unwrap_or(0);
            chatalot_common::api_types::GroupResponse {
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

// ── Timeouts ──

#[derive(serde::Deserialize)]
struct ChannelActionPath {
    cid: Uuid,
    chid: Uuid,
}

#[derive(serde::Deserialize)]
struct ChannelUserPath {
    cid: Uuid,
    chid: Uuid,
    uid: Uuid,
}

/// Verify a channel belongs to the expected community (via its group).
async fn verify_channel_community(
    db: &sqlx::PgPool,
    channel_id: Uuid,
    community_id: Uuid,
) -> Result<(), AppError> {
    if !channel_repo::channel_belongs_to_community(db, channel_id, community_id).await? {
        return Err(AppError::NotFound("channel not found in this community".to_string()));
    }
    Ok(())
}

async fn create_timeout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<ChannelActionPath>,
    Json(req): Json<CreateTimeoutRequest>,
) -> Result<Json<TimeoutResponse>, AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }
    verify_channel_community(&state.db, path.chid, path.cid).await?;

    if req.duration_seconds < 60 || req.duration_seconds > 30 * 24 * 3600 {
        return Err(AppError::Validation(
            "duration must be between 60 seconds and 30 days".into(),
        ));
    }

    let expires_at = chrono::Utc::now()
        + chrono::TimeDelta::seconds(req.duration_seconds);

    let reason = req
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    let id = Uuid::now_v7();
    let timeout = timeout_repo::create(
        &state.db,
        id,
        req.user_id,
        path.chid,
        claims.sub,
        reason,
        expires_at,
    )
    .await?;

    state.connections.broadcast_to_channel(
        path.chid,
        ServerMessage::UserTimedOut {
            channel_id: path.chid,
            user_id: req.user_id,
            expires_at: timeout.expires_at.to_rfc3339(),
            reason: timeout.reason.clone(),
        },
    );

    Ok(Json(TimeoutResponse {
        id: timeout.id,
        user_id: timeout.user_id,
        channel_id: timeout.channel_id,
        issued_by: timeout.issued_by,
        reason: timeout.reason,
        expires_at: timeout.expires_at.to_rfc3339(),
        created_at: timeout.created_at.to_rfc3339(),
    }))
}

async fn remove_timeout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<ChannelUserPath>,
) -> Result<(), AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }
    verify_channel_community(&state.db, path.chid, path.cid).await?;

    if !timeout_repo::remove(&state.db, path.uid, path.chid).await? {
        return Err(AppError::NotFound("no active timeout found".into()));
    }

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "timeout_removed",
        None,
        None,
        Some(serde_json::json!({
            "channel_id": path.chid,
            "target_user_id": path.uid,
        })),
    )
    .await?;

    Ok(())
}

// ── Warnings ──

async fn create_warning(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<ChannelActionPath>,
    Json(req): Json<CreateWarningRequest>,
) -> Result<Json<WarningResponse>, AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }
    verify_channel_community(&state.db, path.chid, path.cid).await?;

    if req.reason.is_empty() || req.reason.len() > 1000 {
        return Err(AppError::Validation(
            "reason must be 1-1000 characters".into(),
        ));
    }

    let id = Uuid::now_v7();
    let warning = warning_repo::create(
        &state.db,
        id,
        req.user_id,
        path.chid,
        claims.sub,
        &req.reason,
    )
    .await?;

    let count = warning_repo::count_for_user_in_channel(&state.db, req.user_id, path.chid).await?;

    state.connections.broadcast_to_channel(
        path.chid,
        ServerMessage::UserWarned {
            channel_id: path.chid,
            user_id: req.user_id,
            reason: req.reason,
            warning_count: count,
        },
    );

    Ok(Json(WarningResponse {
        id: warning.id,
        user_id: warning.user_id,
        channel_id: warning.channel_id,
        issued_by: warning.issued_by,
        reason: warning.reason,
        created_at: warning.created_at.to_rfc3339(),
    }))
}

async fn list_warnings(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<ChannelUserPath>,
) -> Result<Json<Vec<WarningResponse>>, AppError> {
    if !ctx.can_moderate() {
        return Err(AppError::Forbidden);
    }
    verify_channel_community(&state.db, path.chid, path.cid).await?;

    let warnings = warning_repo::list_for_user(&state.db, path.uid, path.chid).await?;
    Ok(Json(
        warnings
            .iter()
            .map(|w| WarningResponse {
                id: w.id,
                user_id: w.user_id,
                channel_id: w.channel_id,
                issued_by: w.issued_by,
                reason: w.reason.clone(),
                created_at: w.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

// ── Community Assets (Icon / Banner) ──

const MAX_COMMUNITY_ICON_SIZE: usize = 2 * 1024 * 1024; // 2MB
const MAX_COMMUNITY_BANNER_SIZE: usize = 5 * 1024 * 1024; // 5MB
const ALLOWED_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

async fn upload_community_icon(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<CommunityResponse>, AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let (data, ct) = read_image_field(&mut multipart, "icon", MAX_COMMUNITY_ICON_SIZE).await?;
    let ext = image_ext(&ct);

    let asset_dir = std::path::Path::new(&state.config.file_storage_path).join("community_assets");
    tokio::fs::create_dir_all(&asset_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create dir: {e}")))?;

    let filename = format!("{}_icon.{ext}", ctx.community_id);
    let file_path = asset_dir.join(&filename);
    write_file(&file_path, &data).await?;

    let icon_url = format!("/api/community-assets/{filename}");
    let community = community_repo::update_community(
        &state.db,
        ctx.community_id,
        None,
        None,
        Some(&icon_url),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("community not found".into()))?;

    let count = community_repo::get_community_member_count(&state.db, ctx.community_id).await?;
    Ok(Json(community_to_response(community, count)))
}

async fn upload_community_banner(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<CommunityResponse>, AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let (data, ct) = read_image_field(&mut multipart, "banner", MAX_COMMUNITY_BANNER_SIZE).await?;
    let ext = image_ext(&ct);

    let asset_dir = std::path::Path::new(&state.config.file_storage_path).join("community_assets");
    tokio::fs::create_dir_all(&asset_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create dir: {e}")))?;

    let filename = format!("{}_banner.{ext}", ctx.community_id);
    let file_path = asset_dir.join(&filename);
    write_file(&file_path, &data).await?;

    let banner_url = format!("/api/community-assets/{filename}");
    let community = community_repo::update_community(
        &state.db,
        ctx.community_id,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&banner_url),
        None,
        None,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("community not found".into()))?;

    let count = community_repo::get_community_member_count(&state.db, ctx.community_id).await?;
    Ok(Json(community_to_response(community, count)))
}

async fn serve_community_asset(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<([(header::HeaderName, String); 2], Body), AppError> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::Validation("invalid filename".into()));
    }

    let path = std::path::Path::new(&state.config.file_storage_path)
        .join("community_assets")
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

fn community_to_response(
    c: chatalot_db::models::community::Community,
    member_count: i64,
) -> CommunityResponse {
    CommunityResponse {
        id: c.id,
        name: c.name,
        description: c.description,
        icon_url: c.icon_url,
        banner_url: c.banner_url,
        owner_id: c.owner_id,
        created_at: c.created_at.to_rfc3339(),
        member_count,
        who_can_create_groups: c.who_can_create_groups,
        who_can_create_invites: c.who_can_create_invites,
        discoverable: c.discoverable,
        community_theme: c.community_theme,
        welcome_message: c.welcome_message,
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
                    "file too large (max {} MB)",
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
            "invalid image type (allowed: png, jpg, webp, gif)".into(),
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

async fn write_file(path: &std::path::Path, data: &[u8]) -> Result<(), AppError> {
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

// ── Custom Emoji ──

const MAX_EMOJI_SIZE: usize = 256 * 1024; // 256KB
const MAX_EMOJIS_PER_COMMUNITY: i64 = 50;
const ALLOWED_EMOJI_TYPES: &[&str] = &["image/png", "image/gif", "image/webp"];

#[derive(serde::Deserialize)]
struct CommunityEmojiPath {
    #[allow(dead_code)]
    cid: Uuid,
    eid: Uuid,
}

async fn list_emojis(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
) -> Result<Json<Vec<CustomEmojiResponse>>, AppError> {
    let emojis = custom_emoji_repo::list_for_community(&state.db, ctx.community_id).await?;
    Ok(Json(
        emojis
            .iter()
            .map(|e| CustomEmojiResponse {
                id: e.id,
                community_id: e.community_id,
                shortcode: e.shortcode.clone(),
                url: format!("/api/emojis/{}", e.id),
                content_type: e.content_type.clone(),
                uploaded_by: e.uploaded_by,
                created_at: e.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn upload_emoji(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Extension(ctx): Extension<CommunityContext>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<CustomEmojiResponse>, AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let count = custom_emoji_repo::count_for_community(&state.db, ctx.community_id).await?;
    if count >= MAX_EMOJIS_PER_COMMUNITY {
        return Err(AppError::Validation(format!(
            "maximum {MAX_EMOJIS_PER_COMMUNITY} emojis per community"
        )));
    }

    let mut file_data: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;
    let mut shortcode: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart error: {e}")))?
    {
        match field.name() {
            Some("file") => {
                content_type = field.content_type().map(|s| s.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("read error: {e}")))?;
                if bytes.len() > MAX_EMOJI_SIZE {
                    return Err(AppError::Validation("emoji too large (max 256KB)".into()));
                }
                file_data = Some(bytes.to_vec());
            }
            Some("shortcode") => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("read error: {e}")))?;
                shortcode = Some(text);
            }
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| AppError::Validation("no file field".into()))?;
    let ct = content_type
        .as_deref()
        .ok_or_else(|| AppError::Validation("missing content type".into()))?;
    let sc = shortcode
        .ok_or_else(|| AppError::Validation("shortcode is required".into()))?
        .trim()
        .to_string();

    if !ALLOWED_EMOJI_TYPES.contains(&ct) {
        return Err(AppError::Validation(
            "invalid image type (allowed: png, gif, webp)".into(),
        ));
    }

    // Validate shortcode: alphanumeric + underscores, 2-32 chars
    if sc.len() < 2 || sc.len() > 32 || !sc.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AppError::Validation(
            "shortcode must be 2-32 alphanumeric/underscore characters".into(),
        ));
    }

    let ext = match ct {
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    };

    let id = Uuid::now_v7();
    let emoji_dir = std::path::Path::new(&state.config.file_storage_path).join("emojis");
    tokio::fs::create_dir_all(&emoji_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create emoji dir: {e}")))?;

    let filename = format!("{id}.{ext}");
    let file_path = emoji_dir.join(&filename);

    let mut f = tokio::fs::File::create(&file_path)
        .await
        .map_err(|e| AppError::Internal(format!("create emoji file: {e}")))?;
    f.write_all(&data)
        .await
        .map_err(|e| AppError::Internal(format!("write emoji: {e}")))?;
    f.flush()
        .await
        .map_err(|e| AppError::Internal(format!("flush emoji: {e}")))?;

    let emoji = custom_emoji_repo::create(
        &state.db,
        id,
        ctx.community_id,
        &sc,
        &file_path.to_string_lossy(),
        ct,
        claims.sub,
    )
    .await?;

    Ok(Json(CustomEmojiResponse {
        id: emoji.id,
        community_id: emoji.community_id,
        shortcode: emoji.shortcode,
        url: format!("/api/emojis/{}", emoji.id),
        content_type: emoji.content_type,
        uploaded_by: emoji.uploaded_by,
        created_at: emoji.created_at.to_rfc3339(),
    }))
}

async fn delete_emoji(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<CommunityContext>,
    Path(path): Path<CommunityEmojiPath>,
) -> Result<(), AppError> {
    if !ctx.can_manage() {
        return Err(AppError::Forbidden);
    }

    let emoji = custom_emoji_repo::get_by_id(&state.db, path.eid)
        .await?
        .ok_or_else(|| AppError::NotFound("emoji not found".into()))?;

    // Delete file from disk
    let _ = tokio::fs::remove_file(&emoji.file_path).await;

    custom_emoji_repo::delete(&state.db, path.eid).await?;
    Ok(())
}

async fn serve_emoji(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<([(header::HeaderName, String); 2], Body), AppError> {
    let emoji = custom_emoji_repo::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("emoji not found".into()))?;

    let file = tokio::fs::File::open(&emoji.file_path)
        .await
        .map_err(|_| AppError::NotFound("emoji file not found".into()))?;

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, emoji.content_type),
            (header::CACHE_CONTROL, "public, max-age=86400".to_string()),
        ],
        body,
    ))
}

// ── Helpers ──

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

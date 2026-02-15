use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use chatalot_common::api_types::{
    AnnouncementResponse, ChangePasswordRequest, DeleteAccountRequest, LogoutAllResponse,
    PreferencesResponse, SessionResponse, UpdatePreferencesRequest, UpdateProfileRequest,
    UserPublic,
};
use chatalot_db::repos::{announcement_repo, group_repo, preferences_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::services::auth_service;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/account/me", get(get_me))
        .route("/account/password", put(change_password))
        .route("/account/profile", put(update_profile))
        .route("/account/avatar", post(upload_avatar))
        .route("/account", delete(delete_account))
        .route("/account/logout-all", post(logout_all))
        .route("/account/sessions", get(list_sessions))
        .route("/account/sessions/{id}", delete(revoke_session))
        .route(
            "/account/preferences",
            get(get_preferences).put(update_preferences),
        )
        .route("/account/announcements", get(get_announcements))
        .route(
            "/account/announcements/{id}/dismiss",
            post(dismiss_announcement),
        )
}

/// Public route for serving avatar images (no auth required).
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new().route("/avatars/{filename}", get(serve_avatar))
}

async fn get_me(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<UserPublic>, AppError> {
    let user = user_repo::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(UserPublic {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        status: user.status,
        custom_status: user.custom_status,
        is_admin: user.is_admin,
        is_owner: user.is_owner,
        created_at: Some(user.created_at.to_rfc3339()),
    }))
}

async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<(), AppError> {
    let user = user_repo::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verify current password
    if !auth_service::verify_password(&req.current_password, &user.password_hash)? {
        return Err(AppError::Validation(
            "current password is incorrect".to_string(),
        ));
    }

    // Validate new password complexity
    auth_service::validate_password(&req.new_password)?;

    // Hash and store
    let new_hash = auth_service::hash_password(&req.new_password)?;
    user_repo::update_password(&state.db, claims.sub, &new_hash).await?;

    // Revoke all refresh tokens (force re-login everywhere)
    user_repo::revoke_all_refresh_tokens(&state.db, claims.sub).await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "password_changed",
        None,
        None,
        None,
    )
    .await?;

    Ok(())
}

async fn update_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserPublic>, AppError> {
    // Validate and trim inputs
    let display_name = req.display_name.as_deref().map(str::trim);
    let custom_status = req.custom_status.as_deref().map(str::trim);
    let bio = req.bio.as_deref().map(str::trim);
    let pronouns = req.pronouns.as_deref().map(str::trim);

    if let Some(dn) = display_name
        && (dn.is_empty() || dn.len() > 64)
    {
        return Err(AppError::Validation(
            "display name must be 1-64 characters".to_string(),
        ));
    }
    if let Some(cs) = custom_status
        && cs.len() > 128
    {
        return Err(AppError::Validation(
            "custom status must be at most 128 characters".to_string(),
        ));
    }

    if let Some(b) = bio
        && b.len() > 500
    {
        return Err(AppError::Validation(
            "bio must be at most 500 characters".to_string(),
        ));
    }
    if let Some(p) = pronouns
        && p.len() > 50
    {
        return Err(AppError::Validation(
            "pronouns must be at most 50 characters".to_string(),
        ));
    }

    let user = user_repo::update_profile(
        &state.db,
        claims.sub,
        display_name,
        req.avatar_url.as_ref().map(|s| Some(s.as_str())),
        custom_status.map(Some),
        bio.map(Some),
        pronouns.map(Some),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    Ok(Json(UserPublic {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        status: user.status,
        custom_status: user.custom_status,
        is_admin: user.is_admin,
        is_owner: user.is_owner,
        created_at: Some(user.created_at.to_rfc3339()),
    }))
}

const MAX_AVATAR_SIZE: usize = 2 * 1024 * 1024; // 2MB
const ALLOWED_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<UserPublic>, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart error: {e}")))?
    {
        if field.name() == Some("avatar") {
            content_type = field.content_type().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("read error: {e}")))?;
            if bytes.len() > MAX_AVATAR_SIZE {
                return Err(AppError::Validation("avatar too large (max 2 MB)".into()));
            }
            file_data = Some(bytes.to_vec());
        }
    }

    let data = file_data.ok_or_else(|| AppError::Validation("no avatar field".into()))?;
    let ct = content_type
        .as_deref()
        .ok_or_else(|| AppError::Validation("missing content type".into()))?;

    if !ALLOWED_TYPES.contains(&ct) {
        return Err(AppError::Validation(
            "invalid image type (allowed: png, jpg, webp, gif)".into(),
        ));
    }

    let ext = match ct {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "bin",
    };

    // Store in avatars/ subdirectory
    let avatar_dir =
        std::path::Path::new(&state.config.file_storage_path).join("avatars");
    tokio::fs::create_dir_all(&avatar_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create avatar dir: {e}")))?;

    let filename = format!("{}.{ext}", claims.sub);
    let file_path = avatar_dir.join(&filename);

    let mut f = tokio::fs::File::create(&file_path)
        .await
        .map_err(|e| AppError::Internal(format!("create avatar file: {e}")))?;
    f.write_all(&data)
        .await
        .map_err(|e| AppError::Internal(format!("write avatar: {e}")))?;
    f.flush()
        .await
        .map_err(|e| AppError::Internal(format!("flush avatar: {e}")))?;

    // Update user's avatar_url to the public serving path
    let avatar_url = format!("/api/avatars/{filename}");
    let user = user_repo::update_profile(
        &state.db,
        claims.sub,
        None,
        Some(Some(&avatar_url)),
        None,
        None,
        None,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    Ok(Json(UserPublic {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        status: user.status,
        custom_status: user.custom_status,
        is_admin: user.is_admin,
        is_owner: user.is_owner,
        created_at: Some(user.created_at.to_rfc3339()),
    }))
}

async fn serve_avatar(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<([(header::HeaderName, String); 2], Body), AppError> {
    // Sanitize filename to prevent path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::Validation("invalid filename".into()));
    }

    let avatar_path = std::path::Path::new(&state.config.file_storage_path)
        .join("avatars")
        .join(&filename);

    let file = tokio::fs::File::open(&avatar_path)
        .await
        .map_err(|_| AppError::NotFound("avatar not found".into()))?;

    let content_type = if filename.ends_with(".png") {
        "image/png"
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".webp") {
        "image/webp"
    } else if filename.ends_with(".gif") {
        "image/gif"
    } else {
        "application/octet-stream"
    };

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (
                header::CACHE_CONTROL,
                "public, max-age=3600".to_string(),
            ),
        ],
        body,
    ))
}

async fn delete_account(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<DeleteAccountRequest>,
) -> Result<(), AppError> {
    let user = user_repo::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verify password
    if !auth_service::verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Validation(
            "password is incorrect".to_string(),
        ));
    }

    // Block if user owns any groups
    if group_repo::user_owns_groups(&state.db, claims.sub).await? {
        return Err(AppError::Validation(
            "you must transfer or delete all groups you own before deleting your account"
                .to_string(),
        ));
    }

    // Audit log before deletion (user_id will become NULL after delete)
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "account_deleted",
        None,
        None,
        None,
    )
    .await?;

    // Delete the user (cascades handle tokens, memberships, keys)
    user_repo::delete_user(&state.db, claims.sub).await?;

    Ok(())
}

async fn logout_all(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<LogoutAllResponse>, AppError> {
    let revoked_count = user_repo::revoke_all_refresh_tokens(&state.db, claims.sub).await?;

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "logout_all",
        None,
        None,
        None,
    )
    .await?;

    Ok(Json(LogoutAllResponse { revoked_count }))
}

async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<SessionResponse>>, AppError> {
    let sessions = user_repo::list_active_sessions(&state.db, claims.sub).await?;

    let responses = sessions
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id,
            device_name: s.device_name,
            ip_address: s.ip_address,
            created_at: s.created_at.to_rfc3339(),
            expires_at: s.expires_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

async fn revoke_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(session_id): Path<Uuid>,
) -> Result<(), AppError> {
    // Verify the session belongs to this user
    let sessions = user_repo::list_active_sessions(&state.db, claims.sub).await?;
    if !sessions.iter().any(|s| s.id == session_id) {
        return Err(AppError::NotFound("session not found".to_string()));
    }

    user_repo::revoke_refresh_token(&state.db, session_id).await?;

    Ok(())
}

async fn get_preferences(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<PreferencesResponse>, AppError> {
    let prefs = preferences_repo::get_preferences(&state.db, claims.sub).await?;
    Ok(Json(PreferencesResponse {
        preferences: prefs,
    }))
}

async fn update_preferences(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<UpdatePreferencesRequest>,
) -> Result<Json<PreferencesResponse>, AppError> {
    // Validate size to prevent abuse (max 16 KB)
    let serialized = serde_json::to_string(&req.preferences)
        .map_err(|_| AppError::Validation("invalid JSON".to_string()))?;
    if serialized.len() > 16_384 {
        return Err(AppError::Validation(
            "preferences too large (max 16 KB)".to_string(),
        ));
    }

    let merged =
        preferences_repo::merge_preferences(&state.db, claims.sub, &req.preferences).await?;
    Ok(Json(PreferencesResponse {
        preferences: merged,
    }))
}

async fn get_announcements(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<AnnouncementResponse>>, AppError> {
    let announcements = announcement_repo::list_undismissed(&state.db, claims.sub).await?;
    Ok(Json(
        announcements
            .iter()
            .map(|a| AnnouncementResponse {
                id: a.id,
                title: a.title.clone(),
                body: a.body.clone(),
                created_by: a.created_by,
                created_at: a.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn dismiss_announcement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(announcement_id): Path<Uuid>,
) -> Result<(), AppError> {
    announcement_repo::dismiss(&state.db, claims.sub, announcement_id).await?;
    Ok(())
}

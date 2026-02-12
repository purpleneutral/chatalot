use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    ChangePasswordRequest, DeleteAccountRequest, LogoutAllResponse, SessionResponse,
    UpdateProfileRequest, UserPublic,
};
use chatalot_db::repos::{group_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::services::auth_service;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/account/password", put(change_password))
        .route("/account/profile", put(update_profile))
        .route("/account", delete(delete_account))
        .route("/account/logout-all", post(logout_all))
        .route("/account/sessions", get(list_sessions))
        .route("/account/sessions/{id}", delete(revoke_session))
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
    // Validate inputs
    if let Some(ref dn) = req.display_name {
        if dn.is_empty() || dn.len() > 64 {
            return Err(AppError::Validation(
                "display name must be 1-64 characters".to_string(),
            ));
        }
    }
    if let Some(ref cs) = req.custom_status {
        if cs.len() > 128 {
            return Err(AppError::Validation(
                "custom status must be at most 128 characters".to_string(),
            ));
        }
    }

    let user = user_repo::update_profile(
        &state.db,
        claims.sub,
        req.display_name.as_deref(),
        req.avatar_url.as_ref().map(|s| Some(s.as_str())),
        req.custom_status.as_ref().map(|s| Some(s.as_str())),
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
    }))
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

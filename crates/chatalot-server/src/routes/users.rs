use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    BlockUserRequest, BlockedUserResponse, CreateReportRequest, ReportResponse, UserPublic,
    UserSearchQuery,
};
use chatalot_db::repos::{block_repo, community_repo, report_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/search", get(search_users))
        .route("/users/{user_id}", get(get_user))
        .route("/users/block", post(block_user))
        .route("/users/unblock/{user_id}", post(unblock_user))
        .route("/users/blocked", get(list_blocked_users))
        .route("/reports", post(create_report))
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPublic>, AppError> {
    let user = user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    // Only visible if caller shares a community with the target (or is instance owner)
    if !claims.is_owner
        && claims.sub != user_id
        && !community_repo::shares_community(&state.db, claims.sub, user_id).await?
    {
        return Err(AppError::NotFound("user not found".to_string()));
    }

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

async fn search_users(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<Vec<UserPublic>>, AppError> {
    if query.q.len() < 2 {
        return Err(AppError::Validation(
            "search query must be at least 2 characters".to_string(),
        ));
    }

    // Instance owner can search all users; everyone else only sees community peers
    let users = if claims.is_owner {
        user_repo::search_users(&state.db, &query.q, 20).await?
    } else {
        community_repo::search_visible_users(&state.db, claims.sub, &query.q, 20).await?
    };

    let results = users
        .iter()
        .map(|u| UserPublic {
            id: u.id,
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            avatar_url: u.avatar_url.clone(),
            status: u.status.clone(),
            custom_status: u.custom_status.clone(),
            is_admin: u.is_admin,
            is_owner: u.is_owner,
            created_at: Some(u.created_at.to_rfc3339()),
        })
        .collect();

    Ok(Json(results))
}

// ── User Blocking ──

async fn block_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<BlockUserRequest>,
) -> Result<(), AppError> {
    if req.user_id == claims.sub {
        return Err(AppError::Validation("cannot block yourself".to_string()));
    }

    // Verify target user exists
    user_repo::find_by_id(&state.db, req.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    block_repo::block_user(&state.db, claims.sub, req.user_id).await?;
    Ok(())
}

async fn unblock_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
) -> Result<(), AppError> {
    if !block_repo::unblock_user(&state.db, claims.sub, user_id).await? {
        return Err(AppError::NotFound("block not found".to_string()));
    }
    Ok(())
}

async fn list_blocked_users(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<BlockedUserResponse>>, AppError> {
    let blocks = block_repo::list_blocked_users(&state.db, claims.sub).await?;
    Ok(Json(
        blocks
            .into_iter()
            .map(|b| BlockedUserResponse {
                blocked_id: b.blocked_id,
                created_at: b.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

// ── Content Reports ──

async fn create_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateReportRequest>,
) -> Result<Json<ReportResponse>, AppError> {
    // Validate report type
    if !["message", "user", "file"].contains(&req.report_type.as_str()) {
        return Err(AppError::Validation(
            "report_type must be 'message', 'user', or 'file'".to_string(),
        ));
    }

    if req.reason.is_empty() || req.reason.len() > 2000 {
        return Err(AppError::Validation(
            "reason must be 1-2000 characters".to_string(),
        ));
    }

    let report = report_repo::create_report(
        &state.db,
        Uuid::now_v7(),
        claims.sub,
        &req.report_type,
        req.target_id,
        &req.reason,
    )
    .await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "report_created",
        None,
        None,
        Some(serde_json::json!({
            "report_id": report.id,
            "report_type": report.report_type,
            "target_id": report.target_id,
        })),
    )
    .await?;

    Ok(Json(report_to_response(report)))
}

fn report_to_response(r: chatalot_db::models::report::Report) -> ReportResponse {
    ReportResponse {
        id: r.id,
        reporter_id: r.reporter_id,
        report_type: r.report_type,
        target_id: r.target_id,
        reason: r.reason,
        status: r.status,
        reviewed_by: r.reviewed_by,
        reviewed_at: r.reviewed_at.map(|t| t.to_rfc3339()),
        admin_notes: r.admin_notes,
        created_at: r.created_at.to_rfc3339(),
    }
}

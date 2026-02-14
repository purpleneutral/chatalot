use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    AdminUserMembership, AdminUserResponse, AdminUsersQuery, CreateRegistrationInviteRequest,
    RegistrationInviteResponse, ResetPasswordRequest, SetAdminRequest, SuspendUserRequest,
};
use chatalot_db::repos::{registration_invite_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::services::auth_service;

/// Guard: returns Forbidden if the caller is not an admin or instance owner.
fn require_admin(claims: &AccessClaims) -> Result<(), AppError> {
    if !claims.is_admin && !claims.is_owner {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

fn user_to_admin_response(user: &chatalot_db::models::user::User) -> AdminUserResponse {
    AdminUserResponse {
        id: user.id,
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        email: user.email.clone(),
        avatar_url: user.avatar_url.clone(),
        is_admin: user.is_admin,
        suspended_at: user.suspended_at.map(|t| t.to_rfc3339()),
        suspended_reason: user.suspended_reason.clone(),
        created_at: user.created_at.to_rfc3339(),
        groups: vec![],
        communities: vec![],
    }
}

#[derive(sqlx::FromRow)]
struct UserMembershipRow {
    user_id: Uuid,
    id: Uuid,
    name: String,
    role: String,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/users/{id}/suspend", post(suspend_user))
        .route("/admin/users/{id}/unsuspend", post(unsuspend_user))
        .route("/admin/users/{id}", delete(delete_user))
        .route("/admin/users/{id}/admin", put(set_admin))
        .route("/admin/users/{id}/password", put(reset_password))
        .route(
            "/admin/invites",
            get(list_registration_invites).post(create_registration_invite),
        )
        .route("/admin/invites/{id}", delete(delete_registration_invite))
}

async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<AdminUsersQuery>,
) -> Result<Json<Vec<AdminUserResponse>>, AppError> {
    require_admin(&claims)?;

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let users =
        user_repo::list_all_users(&state.db, query.search.as_deref(), limit, offset).await?;

    let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();

    // Fetch group memberships for all listed users
    let group_rows: Vec<UserMembershipRow> = sqlx::query_as(
        r#"
        SELECT gm.user_id, g.id, g.name, gm.role
        FROM group_members gm
        INNER JOIN groups g ON g.id = gm.group_id
        WHERE gm.user_id = ANY($1)
        ORDER BY g.name
        "#,
    )
    .bind(&user_ids)
    .fetch_all(&state.db)
    .await?;

    // Fetch community memberships for all listed users
    let community_rows: Vec<UserMembershipRow> = sqlx::query_as(
        r#"
        SELECT cm.user_id, c.id, c.name, cm.role
        FROM community_members cm
        INNER JOIN communities c ON c.id = cm.community_id
        WHERE cm.user_id = ANY($1)
        ORDER BY c.name
        "#,
    )
    .bind(&user_ids)
    .fetch_all(&state.db)
    .await?;

    // Index by user_id
    let mut groups_by_user: std::collections::HashMap<Uuid, Vec<AdminUserMembership>> =
        std::collections::HashMap::new();
    for r in group_rows {
        groups_by_user.entry(r.user_id).or_default().push(AdminUserMembership {
            id: r.id,
            name: r.name,
            role: r.role,
        });
    }
    let mut communities_by_user: std::collections::HashMap<Uuid, Vec<AdminUserMembership>> =
        std::collections::HashMap::new();
    for r in community_rows {
        communities_by_user.entry(r.user_id).or_default().push(AdminUserMembership {
            id: r.id,
            name: r.name,
            role: r.role,
        });
    }

    let responses = users
        .iter()
        .map(|u| {
            let mut resp = user_to_admin_response(u);
            resp.groups = groups_by_user.remove(&u.id).unwrap_or_default();
            resp.communities = communities_by_user.remove(&u.id).unwrap_or_default();
            resp
        })
        .collect();
    Ok(Json(responses))
}

async fn suspend_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SuspendUserRequest>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    // Cannot suspend yourself
    if user_id == claims.sub {
        return Err(AppError::Validation(
            "you cannot suspend yourself".to_string(),
        ));
    }

    // Verify user exists
    let target = user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    // Cannot suspend another admin
    if target.is_admin {
        return Err(AppError::Validation(
            "cannot suspend an admin user".to_string(),
        ));
    }

    user_repo::suspend_user(&state.db, user_id, req.reason.as_deref()).await?;

    // Revoke all their sessions
    user_repo::revoke_all_refresh_tokens(&state.db, user_id).await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_suspend_user",
        None,
        None,
        Some(serde_json::json!({ "target_user_id": user_id })),
    )
    .await?;

    Ok(())
}

async fn unsuspend_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    // Verify user exists
    user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    user_repo::unsuspend_user(&state.db, user_id).await?;

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_unsuspend_user",
        None,
        None,
        Some(serde_json::json!({ "target_user_id": user_id })),
    )
    .await?;

    Ok(())
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    // Cannot delete yourself
    if user_id == claims.sub {
        return Err(AppError::Validation(
            "you cannot delete yourself".to_string(),
        ));
    }

    // Verify user exists
    let target = user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    // Cannot delete another admin
    if target.is_admin {
        return Err(AppError::Validation(
            "cannot delete an admin user; revoke admin first".to_string(),
        ));
    }

    // Audit before deletion
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_delete_user",
        None,
        None,
        Some(serde_json::json!({
            "target_user_id": user_id,
            "target_username": target.username,
        })),
    )
    .await?;

    user_repo::delete_user(&state.db, user_id).await?;

    Ok(())
}

async fn set_admin(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SetAdminRequest>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    // Cannot change your own admin status
    if user_id == claims.sub {
        return Err(AppError::Validation(
            "you cannot change your own admin status".to_string(),
        ));
    }

    // Verify user exists
    user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    user_repo::set_admin(&state.db, user_id, req.is_admin).await?;

    let action = if req.is_admin {
        "admin_grant_admin"
    } else {
        "admin_revoke_admin"
    };

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        action,
        None,
        None,
        Some(serde_json::json!({ "target_user_id": user_id })),
    )
    .await?;

    Ok(())
}

async fn reset_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    if user_id == claims.sub {
        return Err(AppError::Validation(
            "you cannot reset your own password here — use account settings".to_string(),
        ));
    }

    user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    auth_service::validate_password(&req.new_password)?;

    let new_hash = auth_service::hash_password(&req.new_password)?;
    user_repo::update_password(&state.db, user_id, &new_hash).await?;
    user_repo::revoke_all_refresh_tokens(&state.db, user_id).await?;

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_reset_password",
        None,
        None,
        Some(serde_json::json!({ "target_user_id": user_id })),
    )
    .await?;

    Ok(())
}

// ── Registration Invite Management ──

async fn create_registration_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateRegistrationInviteRequest>,
) -> Result<Json<RegistrationInviteResponse>, AppError> {
    require_admin(&claims)?;

    let id = Uuid::now_v7();

    // Generate random 8-char alphanumeric code
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    let code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let expires_at = req.expires_in_hours.map(|h| {
        chrono::Utc::now() + chrono::Duration::try_hours(h).unwrap_or(chrono::Duration::try_hours(24).unwrap())
    });

    let invite = registration_invite_repo::create_invite(
        &state.db,
        id,
        &code,
        claims.sub,
        req.max_uses,
        expires_at,
    )
    .await
    .map_err(|e| AppError::Internal(format!("failed to create invite: {e}")))?;

    Ok(Json(RegistrationInviteResponse {
        id: invite.id,
        code: invite.code,
        created_by: invite.created_by,
        max_uses: invite.max_uses,
        used_count: invite.used_count,
        expires_at: invite.expires_at.map(|t| t.to_rfc3339()),
        created_at: invite.created_at.to_rfc3339(),
    }))
}

async fn list_registration_invites(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<RegistrationInviteResponse>>, AppError> {
    require_admin(&claims)?;

    let invites = registration_invite_repo::list_invites(&state.db).await?;

    let responses = invites
        .into_iter()
        .map(|i| RegistrationInviteResponse {
            id: i.id,
            code: i.code,
            created_by: i.created_by,
            max_uses: i.max_uses,
            used_count: i.used_count,
            expires_at: i.expires_at.map(|t| t.to_rfc3339()),
            created_at: i.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

async fn delete_registration_invite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(invite_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    let deleted = registration_invite_repo::delete_invite(&state.db, invite_id).await?;
    if !deleted {
        return Err(AppError::NotFound("invite not found".to_string()));
    }

    Ok(())
}

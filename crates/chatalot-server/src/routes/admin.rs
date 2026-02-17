use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::HeaderMap;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    AddBlockedHashRequest, AdminFileEntry, AdminFilesQuery, AdminFilesResponse,
    AdminUserMembership, AdminUserResponse, AdminUsersQuery, AnnouncementResponse,
    AuditLogEntryResponse, AuditLogQuery, AuditLogResponse, BlockedHashResponse,
    CreateAnnouncementRequest, CreateRegistrationInviteRequest, PurgeParams, PurgeResult,
    RegistrationInviteResponse, ReportResponse, ReportsQuery, ReportsResponse,
    ResetPasswordRequest, ReviewReportRequest, SetAdminRequest, StorageStatsResponse,
    SuspendUserRequest, UserStorageStatResponse,
};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::models::file::FileRecord;
use chatalot_db::repos::{
    announcement_repo, audit_repo, blocked_hash_repo, file_repo, message_repo,
    registration_invite_repo, report_repo, user_repo,
};

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
        // User management
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
        // Purge endpoints
        .route("/admin/purge/message/{id}", post(purge_message))
        .route("/admin/purge/user/{id}/messages", post(purge_user_messages))
        .route("/admin/purge/channel/{id}", post(purge_channel))
        // File management
        .route("/admin/files", get(list_all_files))
        .route("/admin/files/{id}", delete(admin_delete_file))
        .route("/admin/files/{id}/quarantine", post(quarantine_file))
        .route("/admin/files/{id}/unquarantine", post(unquarantine_file))
        .route("/admin/storage-stats", get(storage_stats))
        // Message quarantine
        .route("/admin/messages/{id}/quarantine", post(quarantine_message))
        .route(
            "/admin/messages/{id}/unquarantine",
            post(unquarantine_message),
        )
        // Hash blocklist
        .route(
            "/admin/blocked-hashes",
            get(list_blocked_hashes).post(add_blocked_hash),
        )
        .route("/admin/blocked-hashes/{id}", delete(remove_blocked_hash))
        // Audit log
        .route("/admin/audit-log", get(query_audit_log))
        // Reports
        .route("/admin/reports", get(list_reports))
        .route("/admin/reports/{id}/review", post(review_report))
        // Announcements
        .route(
            "/admin/announcements",
            post(create_announcement).get(list_announcements),
        )
}

// ── User Management (existing) ──

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
        groups_by_user
            .entry(r.user_id)
            .or_default()
            .push(AdminUserMembership {
                id: r.id,
                name: r.name,
                role: r.role,
            });
    }
    let mut communities_by_user: std::collections::HashMap<Uuid, Vec<AdminUserMembership>> =
        std::collections::HashMap::new();
    for r in community_rows {
        communities_by_user
            .entry(r.user_id)
            .or_default()
            .push(AdminUserMembership {
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

    if let Some(ref r) = req.reason
        && r.len() > 500
    {
        return Err(AppError::Validation(
            "reason must be at most 500 characters".to_string(),
        ));
    }

    user_repo::suspend_user(&state.db, user_id, req.reason.as_deref()).await?;
    state.suspended_users.insert(user_id);

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
    state.suspended_users.remove(&user_id);

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
    // Only the instance owner can promote/demote admins
    if !claims.is_owner {
        return Err(AppError::Forbidden);
    }

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
    conn_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
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

    let ip = super::auth::extract_client_ip(&headers, Some(conn_info.0));
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .filter(|ua| ua.len() <= 512);

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_reset_password",
        ip.as_deref(),
        ua,
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

    // Generate random 12-char alphanumeric code
    use rand::Rng;
    use rand::distributions::Alphanumeric;
    let code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    if let Some(h) = req.expires_in_hours
        && !(1..=8760).contains(&h)
    {
        return Err(AppError::Validation(
            "expires_in_hours must be between 1 and 8760".into(),
        ));
    }
    let expires_at = req.expires_in_hours.map(|h| {
        chrono::Utc::now()
            + chrono::Duration::try_hours(h).unwrap_or(chrono::TimeDelta::hours(24))
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

// ── Purge Endpoints ──

/// Helper: delete file blobs from disk, return count of successful deletions.
async fn delete_files_from_disk(files: &[FileRecord]) -> u64 {
    let mut count = 0u64;
    for file in files {
        match tokio::fs::remove_file(&file.storage_path).await {
            Ok(()) => count += 1,
            Err(e) => {
                tracing::warn!(
                    file_id = %file.id,
                    path = %file.storage_path,
                    "Failed to delete file from disk: {e}"
                );
            }
        }
    }
    count
}

/// Helper: optionally block file hashes and return count of hashes blocked.
async fn maybe_block_hashes(
    db: &sqlx::PgPool,
    files: &[FileRecord],
    block: bool,
    blocked_by: Uuid,
) -> u64 {
    if !block || files.is_empty() {
        return 0;
    }
    let hashes: Vec<String> = files.iter().map(|f| f.checksum.clone()).collect();
    blocked_hash_repo::add_blocked_hashes(
        db,
        &hashes,
        Some("auto-blocked via admin purge"),
        blocked_by,
    )
    .await
    .unwrap_or(0)
}

/// Hard-delete a single message and its sender's files from disk.
async fn purge_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(message_id): Path<Uuid>,
    Query(params): Query<PurgeParams>,
) -> Result<Json<PurgeResult>, AppError> {
    require_admin(&claims)?;

    let block_hashes = params.block_hashes.unwrap_or(false);

    // Get message info before deletion
    let msg = message_repo::get_message_by_id(&state.db, message_id)
        .await?
        .ok_or_else(|| AppError::NotFound("message not found".to_string()))?;

    // Hard-delete the message
    message_repo::hard_delete_message(&state.db, message_id).await?;

    // Note: single message purge does not auto-delete files because the server
    // can't know which file the message references (E2E encrypted content).
    // Use the file purge endpoints separately.

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_purge_message",
        None,
        None,
        Some(serde_json::json!({
            "message_id": message_id,
            "channel_id": msg.channel_id,
            "block_hashes": block_hashes,
        })),
    )
    .await?;

    Ok(Json(PurgeResult {
        messages_deleted: 1,
        files_deleted: 0,
        hashes_blocked: 0,
    }))
}

/// Hard-delete ALL messages from a user + all their uploaded files.
async fn purge_user_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<PurgeParams>,
) -> Result<Json<PurgeResult>, AppError> {
    require_admin(&claims)?;

    let block_hashes = params.block_hashes.unwrap_or(false);

    // Get all files uploaded by this user (before deleting DB records)
    let files = file_repo::list_all_user_files(&state.db, user_id).await?;

    // Optionally block hashes
    let hashes_blocked = maybe_block_hashes(&state.db, &files, block_hashes, claims.sub).await;

    // Delete files from disk
    let files_deleted = delete_files_from_disk(&files).await;

    // Delete file DB records
    file_repo::delete_all_user_files(&state.db, user_id).await?;

    // Hard-delete all messages
    let messages_deleted = message_repo::hard_delete_user_messages(&state.db, user_id).await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_purge_user_messages",
        None,
        None,
        Some(serde_json::json!({
            "target_user_id": user_id,
            "messages_deleted": messages_deleted,
            "files_deleted": files_deleted,
            "hashes_blocked": hashes_blocked,
        })),
    )
    .await?;

    Ok(Json(PurgeResult {
        messages_deleted,
        files_deleted,
        hashes_blocked,
    }))
}

/// Hard-delete ALL messages in a channel + associated files.
async fn purge_channel(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
    Query(params): Query<PurgeParams>,
) -> Result<Json<PurgeResult>, AppError> {
    require_admin(&claims)?;

    let block_hashes = params.block_hashes.unwrap_or(false);

    // Get all files in this channel
    let files = file_repo::list_channel_files(&state.db, channel_id).await?;

    // Optionally block hashes
    let hashes_blocked = maybe_block_hashes(&state.db, &files, block_hashes, claims.sub).await;

    // Delete files from disk
    let files_deleted = delete_files_from_disk(&files).await;

    // Delete file DB records
    file_repo::delete_channel_files(&state.db, channel_id).await?;

    // Hard-delete all messages
    let messages_deleted =
        message_repo::hard_delete_channel_messages(&state.db, channel_id).await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_purge_channel",
        None,
        None,
        Some(serde_json::json!({
            "channel_id": channel_id,
            "messages_deleted": messages_deleted,
            "files_deleted": files_deleted,
            "hashes_blocked": hashes_blocked,
        })),
    )
    .await?;

    Ok(Json(PurgeResult {
        messages_deleted,
        files_deleted,
        hashes_blocked,
    }))
}

// ── File Management ──

/// List all files (admin browser) with pagination and sorting.
async fn list_all_files(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<AdminFilesQuery>,
) -> Result<Json<AdminFilesResponse>, AppError> {
    require_admin(&claims)?;

    let page = query.page.unwrap_or(1).clamp(1, 10_000);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let offset = (page - 1) * per_page;
    let sort = query.sort.as_deref().unwrap_or("date");

    let total = file_repo::count_files(&state.db, query.user_id).await?;
    let files = file_repo::list_all_files(&state.db, query.user_id, sort, per_page, offset).await?;

    let entries = files
        .into_iter()
        .map(|f| AdminFileEntry {
            id: f.id,
            uploader_id: f.uploader_id,
            encrypted_name: f.encrypted_name,
            size_bytes: f.size_bytes,
            content_type: f.content_type,
            checksum: f.checksum,
            channel_id: f.channel_id,
            quarantined_at: f.quarantined_at.map(|t| t.to_rfc3339()),
            quarantined_by: f.quarantined_by,
            created_at: f.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(AdminFilesResponse {
        files: entries,
        total,
        page,
        per_page,
    }))
}

/// Admin delete any file + optional hash blocking.
async fn admin_delete_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(file_id): Path<Uuid>,
    Query(params): Query<PurgeParams>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    let record = file_repo::get_file(&state.db, file_id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".to_string()))?;

    // Optionally block the hash
    if params.block_hashes.unwrap_or(false) {
        let _ = blocked_hash_repo::add_blocked_hash(
            &state.db,
            Uuid::now_v7(),
            &record.checksum,
            Some("blocked via admin file delete"),
            claims.sub,
        )
        .await;
    }

    // Delete from disk
    if let Err(e) = tokio::fs::remove_file(&record.storage_path).await {
        tracing::warn!(
            "Failed to delete file from disk {}: {e}",
            record.storage_path
        );
    }

    // Delete from DB
    file_repo::delete_file_admin(&state.db, file_id).await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_delete_file",
        None,
        None,
        Some(serde_json::json!({
            "file_id": file_id,
            "uploader_id": record.uploader_id,
            "checksum": record.checksum,
        })),
    )
    .await?;

    Ok(())
}

/// Get storage statistics.
async fn storage_stats(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<StorageStatsResponse>, AppError> {
    require_admin(&claims)?;

    let stats = file_repo::storage_stats(&state.db).await?;
    let total_files: i64 = stats.iter().map(|s| s.file_count).sum();
    let total_bytes: i64 = stats.iter().map(|s| s.total_bytes).sum();

    let per_user = stats
        .into_iter()
        .map(|s| UserStorageStatResponse {
            user_id: s.uploader_id,
            file_count: s.file_count,
            total_bytes: s.total_bytes,
        })
        .collect();

    Ok(Json(StorageStatsResponse {
        total_files,
        total_bytes,
        per_user,
    }))
}

// ── Quarantine ──

/// Quarantine a file (hide from downloads, preserve for evidence).
async fn quarantine_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(file_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    let updated = file_repo::quarantine_file(&state.db, file_id, claims.sub).await?;
    if !updated {
        return Err(AppError::NotFound(
            "file not found or already quarantined".to_string(),
        ));
    }

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_quarantine_file",
        None,
        None,
        Some(serde_json::json!({ "file_id": file_id })),
    )
    .await?;

    Ok(())
}

/// Lift quarantine from a file.
async fn unquarantine_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(file_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    let updated = file_repo::unquarantine_file(&state.db, file_id).await?;
    if !updated {
        return Err(AppError::NotFound(
            "file not found or not quarantined".to_string(),
        ));
    }

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_unquarantine_file",
        None,
        None,
        Some(serde_json::json!({ "file_id": file_id })),
    )
    .await?;

    Ok(())
}

/// Quarantine a message (hide from channel, preserve for evidence).
async fn quarantine_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(message_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    let updated = message_repo::quarantine_message(&state.db, message_id, claims.sub).await?;
    if !updated {
        return Err(AppError::NotFound(
            "message not found or already quarantined".to_string(),
        ));
    }

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_quarantine_message",
        None,
        None,
        Some(serde_json::json!({ "message_id": message_id })),
    )
    .await?;

    Ok(())
}

/// Lift quarantine from a message.
async fn unquarantine_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(message_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    let updated = message_repo::unquarantine_message(&state.db, message_id).await?;
    if !updated {
        return Err(AppError::NotFound(
            "message not found or not quarantined".to_string(),
        ));
    }

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_unquarantine_message",
        None,
        None,
        Some(serde_json::json!({ "message_id": message_id })),
    )
    .await?;

    Ok(())
}

// ── Hash Blocklist ──

/// List blocked hashes.
async fn list_blocked_hashes(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<AdminFilesQuery>,
) -> Result<Json<Vec<BlockedHashResponse>>, AppError> {
    require_admin(&claims)?;

    let limit = query.per_page.unwrap_or(50).min(100);
    let page = query.page.unwrap_or(1).clamp(1, 10_000);
    let offset = (page - 1) * limit;

    let hashes = blocked_hash_repo::list_blocked_hashes(&state.db, limit, offset).await?;

    let responses = hashes
        .into_iter()
        .map(|h| BlockedHashResponse {
            id: h.id,
            hash: h.hash,
            reason: h.reason,
            blocked_by: h.blocked_by,
            created_at: h.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

/// Add a hash to the blocklist.
async fn add_blocked_hash(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<AddBlockedHashRequest>,
) -> Result<Json<BlockedHashResponse>, AppError> {
    require_admin(&claims)?;

    if req.hash.len() != 64 {
        return Err(AppError::Validation(
            "hash must be a 64-character hex SHA256".to_string(),
        ));
    }

    if let Some(ref r) = req.reason
        && r.len() > 500
    {
        return Err(AppError::Validation(
            "reason must be at most 500 characters".to_string(),
        ));
    }

    let record = blocked_hash_repo::add_blocked_hash(
        &state.db,
        Uuid::now_v7(),
        &req.hash,
        req.reason.as_deref(),
        claims.sub,
    )
    .await?;

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_block_hash",
        None,
        None,
        Some(serde_json::json!({
            "hash": req.hash,
            "reason": req.reason,
        })),
    )
    .await?;

    Ok(Json(BlockedHashResponse {
        id: record.id,
        hash: record.hash,
        reason: record.reason,
        blocked_by: record.blocked_by,
        created_at: record.created_at.to_rfc3339(),
    }))
}

/// Remove a hash from the blocklist.
async fn remove_blocked_hash(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(hash_id): Path<Uuid>,
) -> Result<(), AppError> {
    require_admin(&claims)?;

    let deleted = blocked_hash_repo::remove_blocked_hash(&state.db, hash_id).await?;
    if !deleted {
        return Err(AppError::NotFound("blocked hash not found".to_string()));
    }

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "admin_unblock_hash",
        None,
        None,
        Some(serde_json::json!({ "hash_id": hash_id })),
    )
    .await?;

    Ok(())
}

// ── Audit Log ──

/// Query audit log with optional filters.
async fn query_audit_log(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<AuditLogResponse>, AppError> {
    require_admin(&claims)?;

    let page = query.page.unwrap_or(1).clamp(1, 10_000);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let offset = (page - 1) * per_page;

    let total =
        audit_repo::count_audit_log(&state.db, query.action.as_deref(), query.user_id).await?;

    let entries = audit_repo::query_audit_log(
        &state.db,
        query.action.as_deref(),
        query.user_id,
        per_page,
        offset,
    )
    .await?;

    let responses = entries
        .into_iter()
        .map(|e| AuditLogEntryResponse {
            id: e.id,
            user_id: e.user_id,
            action: e.action,
            ip_address: e.ip_address,
            user_agent: e.user_agent,
            metadata: e.metadata,
            created_at: e.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(AuditLogResponse {
        entries: responses,
        total,
        page,
        per_page,
    }))
}

// ── Reports ──

/// List reports with optional status filter and pagination.
async fn list_reports(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<ReportsQuery>,
) -> Result<Json<ReportsResponse>, AppError> {
    require_admin(&claims)?;

    let page = query.page.unwrap_or(1).clamp(1, 10_000);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let offset = (page - 1) * per_page;

    let total = report_repo::count_reports(&state.db, query.status.as_deref()).await?;
    let reports =
        report_repo::list_reports(&state.db, query.status.as_deref(), per_page, offset).await?;

    let responses = reports
        .into_iter()
        .map(|r| ReportResponse {
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
        })
        .collect();

    Ok(Json(ReportsResponse {
        reports: responses,
        total,
        page,
        per_page,
    }))
}

/// Review a report (change status, add admin notes).
async fn review_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(report_id): Path<Uuid>,
    Json(req): Json<ReviewReportRequest>,
) -> Result<Json<ReportResponse>, AppError> {
    require_admin(&claims)?;

    if !["reviewed", "resolved", "dismissed"].contains(&req.status.as_str()) {
        return Err(AppError::Validation(
            "status must be 'reviewed', 'resolved', or 'dismissed'".to_string(),
        ));
    }

    if let Some(ref notes) = req.admin_notes
        && notes.len() > 5000
    {
        return Err(AppError::Validation(
            "admin notes must be at most 5000 characters".to_string(),
        ));
    }

    let report = report_repo::review_report(
        &state.db,
        report_id,
        claims.sub,
        &req.status,
        req.admin_notes.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("report not found".to_string()))?;

    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "report_reviewed",
        None,
        None,
        Some(serde_json::json!({
            "report_id": report_id,
            "new_status": req.status,
        })),
    )
    .await?;

    Ok(Json(ReportResponse {
        id: report.id,
        reporter_id: report.reporter_id,
        report_type: report.report_type,
        target_id: report.target_id,
        reason: report.reason,
        status: report.status,
        reviewed_by: report.reviewed_by,
        reviewed_at: report.reviewed_at.map(|t| t.to_rfc3339()),
        admin_notes: report.admin_notes,
        created_at: report.created_at.to_rfc3339(),
    }))
}

// ── Announcements ──

async fn create_announcement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateAnnouncementRequest>,
) -> Result<Json<AnnouncementResponse>, AppError> {
    require_admin(&claims)?;

    if req.title.is_empty() || req.title.len() > 200 {
        return Err(AppError::Validation(
            "title must be 1-200 characters".into(),
        ));
    }
    if req.body.is_empty() || req.body.len() > 5000 {
        return Err(AppError::Validation(
            "body must be 1-5000 characters".into(),
        ));
    }

    let id = Uuid::now_v7();
    let ann = announcement_repo::create(&state.db, id, &req.title, &req.body, claims.sub).await?;

    // Broadcast to all connected users
    state
        .connections
        .broadcast_all(ServerMessage::Announcement {
            id: ann.id,
            title: ann.title.clone(),
            body: ann.body.clone(),
            created_by: ann.created_by,
            created_at: ann.created_at.to_rfc3339(),
        });

    Ok(Json(AnnouncementResponse {
        id: ann.id,
        title: ann.title,
        body: ann.body,
        created_by: ann.created_by,
        created_at: ann.created_at.to_rfc3339(),
    }))
}

async fn list_announcements(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<AnnouncementResponse>>, AppError> {
    require_admin(&claims)?;

    let announcements = announcement_repo::list_all(&state.db).await?;
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

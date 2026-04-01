use std::sync::Arc;

use axum::extract::{ConnectInfo, Extension, Query, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::Instant;
use uuid::Uuid;
use zeroize::Zeroize;

use chatalot_common::api_types::{SignedPrekeyUpload, OneTimePrekeyUpload};
use chatalot_common::constants::{REFRESH_TOKEN_LIFETIME_SECS};
use chatalot_db::repos::{key_repo, oidc_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::services::auth_service;

// ── Rate Limiting ──

/// Simple in-memory rate limiter for the callback endpoint.
/// Tracks (IP -> (count, window_start)).
static CALLBACK_RATE_LIMIT: LazyLock<DashMap<String, (u32, Instant)>> = LazyLock::new(DashMap::new);

/// Max callback requests per IP per window.
const CALLBACK_MAX_REQUESTS: u32 = 10;
/// Rate limit window duration.
const CALLBACK_WINDOW_SECS: u64 = 60;

fn check_callback_rate_limit(ip: &str) -> Result<(), AppError> {
    let now = Instant::now();
    let mut entry = CALLBACK_RATE_LIMIT
        .entry(ip.to_string())
        .or_insert_with(|| (0, now));

    // Reset window if expired
    if entry.1.elapsed().as_secs() > CALLBACK_WINDOW_SECS {
        entry.0 = 0;
        entry.1 = now;
    }

    entry.0 += 1;
    if entry.0 > CALLBACK_MAX_REQUESTS {
        return Err(AppError::Validation("too many requests, try again later".to_string()));
    }
    Ok(())
}

// ── Request/Response Types ──

#[derive(Debug, Serialize)]
pub struct OidcInitiateResponse {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OidcAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: chatalot_common::api_types::UserPublic,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_code: Option<String>,
    pub keys_registered: bool,
}

#[derive(Debug, Deserialize)]
pub struct OidcCompleteSetupRequest {
    /// Ed25519 public identity key (32 bytes)
    pub identity_key: Vec<u8>,
    /// Signed prekey bundle
    pub signed_prekey: SignedPrekeyUpload,
    /// Initial batch of one-time prekeys
    pub one_time_prekeys: Vec<OneTimePrekeyUpload>,
}

// ── Routes ──

/// Public OIDC routes (no auth required).
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/oidc/initiate", get(initiate))
        .route("/auth/oidc/callback", get(callback))
}

/// Authenticated OIDC routes (JWT required).
pub fn authenticated_routes() -> Router<Arc<AppState>> {
    Router::new().route("/auth/oidc/complete-setup", post(complete_setup))
}

/// GET /api/auth/oidc/initiate — generate OIDC authorization URL.
async fn initiate(
    State(state): State<Arc<AppState>>,
) -> Result<Json<OidcInitiateResponse>, AppError> {
    let oidc = state
        .oidc_service
        .as_ref()
        .ok_or_else(|| AppError::Validation("SSO is not configured".to_string()))?;

    let (url, _state_key) = oidc.generate_auth_url();

    Ok(Json(OidcInitiateResponse { url }))
}

/// GET /api/auth/oidc/callback?code=...&state=... — exchange code for tokens.
async fn callback(
    State(state): State<Arc<AppState>>,
    conn_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<OidcCallbackQuery>,
) -> Result<Json<OidcAuthResponse>, AppError> {
    let ip = crate::routes::auth::extract_client_ip(&headers, Some(conn_info.0));

    // Rate limit
    if let Some(ref ip_str) = ip {
        check_callback_rate_limit(ip_str)?;
    }

    let oidc = state
        .oidc_service
        .as_ref()
        .ok_or_else(|| AppError::Validation("SSO is not configured".to_string()))?;

    // Validate input lengths
    if query.code.len() > 2048 || query.state.len() > 256 {
        return Err(AppError::Validation("invalid request parameters".to_string()));
    }

    // Exchange code for user info
    let user_info = oidc
        .exchange_code(&query.code, &query.state)
        .await
        .map_err(|e| {
            tracing::warn!("OIDC code exchange failed: {e}");
            AppError::Unauthorized
        })?;

    let provider = "authentik"; // Hardcoded for now; can be made dynamic later
    let device_name = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .filter(|ua| ua.len() <= 512)
        .map(|ua| {
            if ua.len() <= 50 {
                ua.to_string()
            } else {
                ua.chars().take(50).collect()
            }
        });

    // Step 1: Check if OIDC identity is already linked
    if let Some((user_id, _username)) =
        oidc_repo::find_by_provider_subject(&state.db, provider, &user_info.subject).await?
    {
        let user = user_repo::find_by_id(&state.db, user_id)
            .await?
            .ok_or(AppError::Internal("linked user not found".to_string()))?;

        if user.suspended_at.is_some() {
            return Err(AppError::Validation("account is suspended".to_string()));
        }

        return issue_oidc_tokens(&state, &user, device_name.as_deref(), ip.as_deref()).await;
    }

    // Step 2: Check if email matches an existing user (link identity)
    if let Some(ref email) = user_info.email {
        if let Some(user) = user_repo::find_by_email(&state.db, email).await? {
            if user.suspended_at.is_some() {
                return Err(AppError::Validation("account is suspended".to_string()));
            }

            // Link OIDC identity to existing user
            oidc_repo::link_identity(
                &state.db,
                user.id,
                provider,
                &user_info.subject,
                Some(email.as_str()),
            )
            .await?;

            // Audit log
            user_repo::insert_audit_log(
                &state.db,
                Uuid::now_v7(),
                Some(user.id),
                "oidc_identity_linked",
                ip.as_deref(),
                device_name.as_deref(),
                None,
            )
            .await?;

            return issue_oidc_tokens(&state, &user, device_name.as_deref(), ip.as_deref()).await;
        }
    }

    // Step 3: Create a new user
    let username = generate_unique_username(
        &state.db,
        user_info.preferred_username.as_deref(),
        &user_info.subject,
    )
    .await?;

    let display_name = user_info
        .name
        .clone()
        .or_else(|| user_info.preferred_username.clone())
        .unwrap_or_else(|| username.clone());

    let email = user_info
        .email
        .clone()
        .unwrap_or_else(|| format!("{}@oidc.local", user_info.subject));

    let user_id = Uuid::now_v7();
    let user = user_repo::create_oidc_user(
        &state.db,
        user_id,
        &username,
        &email,
        &display_name,
        provider,
        &user_info.subject,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create OIDC user: {e}");
        AppError::Internal("failed to create user account".to_string())
    })?;

    // Link OIDC identity
    oidc_repo::link_identity(
        &state.db,
        user.id,
        provider,
        &user_info.subject,
        user_info.email.as_deref(),
    )
    .await?;

    // First registered user becomes admin + owner
    if user_repo::count_users(&state.db).await.unwrap_or(1) == 1 {
        user_repo::set_admin(&state.db, user.id, true).await.ok();
        user_repo::set_owner(&state.db, user.id, true).await.ok();
    }

    // Generate recovery code
    let (recovery_code, recovery_hash) = auth_service::generate_recovery_code();
    user_repo::set_recovery_code_hash(&state.db, user.id, &recovery_hash).await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(user.id),
        "oidc_register",
        ip.as_deref(),
        device_name.as_deref(),
        None,
    )
    .await?;

    let mut response =
        issue_oidc_tokens(&state, &user, device_name.as_deref(), ip.as_deref()).await?;
    response.recovery_code = Some(recovery_code);

    Ok(response)
}

/// POST /api/auth/oidc/complete-setup — register E2E keys for OIDC user.
async fn complete_setup(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<OidcCompleteSetupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate key lengths
    if req.identity_key.len() != 32 {
        return Err(AppError::Validation(
            "identity key must be 32 bytes".to_string(),
        ));
    }
    if req.signed_prekey.public_key.len() != 32 {
        return Err(AppError::Validation(
            "signed prekey must be 32 bytes".to_string(),
        ));
    }
    if req.signed_prekey.signature.len() != 64 {
        return Err(AppError::Validation(
            "signed prekey signature must be 64 bytes".to_string(),
        ));
    }

    // Verify the signed prekey signature against the identity key
    auth_service::verify_signed_prekey_signature(
        &req.identity_key,
        &req.signed_prekey.public_key,
        &req.signed_prekey.signature,
    )?;

    // Compute identity key fingerprint
    let fingerprint = hex::encode(Sha256::digest(&req.identity_key));

    // Upsert identity key
    key_repo::upsert_identity_key(&state.db, claims.sub, &req.identity_key, &fingerprint).await?;

    // Store signed prekey
    key_repo::upsert_signed_prekey(
        &state.db,
        Uuid::now_v7(),
        claims.sub,
        req.signed_prekey.key_id,
        &req.signed_prekey.public_key,
        &req.signed_prekey.signature,
    )
    .await?;

    // Store one-time prekeys
    if !req.one_time_prekeys.is_empty() {
        let pairs: Vec<(i32, Vec<u8>)> = req
            .one_time_prekeys
            .into_iter()
            .map(|p| (p.key_id, p.public_key))
            .collect();
        key_repo::upload_one_time_prekeys(&state.db, claims.sub, &pairs).await?;
    }

    // Mark keys as registered
    user_repo::set_keys_registered(&state.db, claims.sub).await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "oidc_keys_registered",
        None,
        None,
        None,
    )
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ── Helpers ──

/// Issue JWT tokens for an OIDC-authenticated user.
async fn issue_oidc_tokens(
    state: &AppState,
    user: &chatalot_db::models::user::User,
    device_name: Option<&str>,
    ip_address: Option<&str>,
) -> Result<Json<OidcAuthResponse>, AppError> {
    let (is_admin, is_owner) = (user.is_admin, user.is_owner);

    // Issue access token
    let access_token =
        auth_service::issue_access_token(state, user.id, &user.username, is_admin, is_owner)?;

    // Generate refresh token
    let (mut refresh_raw, refresh_hash) = auth_service::generate_refresh_token();
    let refresh_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::TimeDelta::seconds(REFRESH_TOKEN_LIFETIME_SECS);

    user_repo::create_refresh_token(
        &state.db,
        refresh_id,
        user.id,
        &refresh_hash,
        device_name,
        ip_address,
        expires_at,
    )
    .await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(user.id),
        "oidc_login",
        ip_address,
        device_name,
        None,
    )
    .await?;

    let refresh_token_hex = hex::encode(&refresh_raw);
    refresh_raw.zeroize();

    let user_public = auth_service::user_to_public(user, is_admin, is_owner);

    Ok(Json(OidcAuthResponse {
        access_token,
        refresh_token: refresh_token_hex,
        user: user_public,
        recovery_code: None,
        keys_registered: user.keys_registered,
    }))
}

/// Generate a unique username from OIDC claims, appending a suffix if needed.
async fn generate_unique_username(
    pool: &sqlx::PgPool,
    preferred: Option<&str>,
    subject: &str,
) -> Result<String, AppError> {
    // Sanitize the preferred username
    let base = preferred
        .map(|u| sanitize_username(u))
        .filter(|u| u.len() >= 3)
        .unwrap_or_else(|| {
            // Fall back to a sanitized subject or random
            let sanitized = sanitize_username(subject);
            if sanitized.len() >= 3 {
                sanitized
            } else {
                format!("user_{}", &Uuid::new_v4().to_string()[..8])
            }
        });

    // Truncate to 28 chars to leave room for suffix
    let base = if base.len() > 28 {
        base[..28].to_string()
    } else {
        base
    };

    // Try the base username first
    if !user_repo::username_exists(pool, &base).await? {
        return Ok(base);
    }

    // Try with numeric suffixes
    for i in 1..100 {
        let candidate = format!("{base}_{i}");
        if !user_repo::username_exists(pool, &candidate).await? {
            return Ok(candidate);
        }
    }

    // Ultimate fallback
    Ok(format!("user_{}", &Uuid::new_v4().to_string()[..8]))
}

/// Sanitize a string to be a valid username (alphanumeric + underscore, 3-32 chars).
fn sanitize_username(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .take(32)
        .collect();

    // Ensure it starts with alphanumeric
    let sanitized = sanitized
        .trim_start_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_string();

    // Remove trailing dots
    sanitized.trim_end_matches('.').to_string()
}

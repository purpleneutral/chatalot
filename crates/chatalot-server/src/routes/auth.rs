use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};

use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use chatalot_common::api_types::{
    AuthResponse, LoginRequest, RecoverAccountRequest, RecoverAccountResponse, RefreshRequest,
    RegisterRequest, ServerConfigResponse, TokenResponse,
};
use chatalot_db::repos::user_repo;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::auth_service;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/recover", post(recover_account))
        .route("/auth/config", get(server_config))
}

async fn server_config(State(state): State<Arc<AppState>>) -> Json<ServerConfigResponse> {
    let ice_servers = state.config.ice_servers_json.as_ref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    Json(ServerConfigResponse {
        registration_mode: state.config.registration_mode.clone(),
        public_url: state.config.public_url.clone(),
        ice_servers,
    })
}

/// Extract a human-readable device name from a User-Agent string.
fn parse_device_name(ua: &str) -> String {
    // Tauri desktop client
    if ua.contains("Tauri") || ua.contains("tauri") {
        return "Chatalot Desktop".to_string();
    }

    // Detect browser
    let browser = if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("Chrome/") && !ua.contains("Chromium/") {
        "Chrome"
    } else if ua.contains("Firefox/") {
        "Firefox"
    } else if ua.contains("Safari/") && !ua.contains("Chrome/") {
        "Safari"
    } else if ua.contains("Chromium/") {
        "Chromium"
    } else {
        ""
    };

    // Detect OS
    let os = if ua.contains("Android") {
        "Android"
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS"
    } else if ua.contains("Linux") {
        "Linux"
    } else if ua.contains("Mac OS") || ua.contains("Macintosh") {
        "macOS"
    } else if ua.contains("Windows") {
        "Windows"
    } else {
        ""
    };

    match (browser, os) {
        ("", "") => {
            // Fallback: truncate raw UA
            let truncated: String = ua.chars().take(50).collect();
            truncated
        }
        ("", os) => os.to_string(),
        (browser, "") => browser.to_string(),
        (browser, os) => format!("{browser} on {os}"),
    }
}

/// Extract client IP, trusting proxy headers only from trusted connections.
pub(crate) fn extract_client_ip(
    headers: &HeaderMap,
    conn_ip: Option<SocketAddr>,
) -> Option<String> {
    let peer_ip = conn_ip.map(|c| c.ip());

    // Only trust proxy headers when the connection comes from a trusted proxy
    let trust_headers = peer_ip.is_some_and(|ip| {
        use std::net::IpAddr;
        match ip {
            IpAddr::V4(v4) => {
                v4.is_loopback()
                    || v4.octets()[..2] == [172, 17]
                    || v4.octets()[..2] == [172, 18]
                    || v4.octets()[..3] == [172, 19, 0]
            }
            IpAddr::V6(v6) => v6.is_loopback(),
        }
    });

    if trust_headers {
        // X-Forwarded-For (first entry is the client)
        if let Some(xff) = headers.get("x-forwarded-for")
            && let Ok(val) = xff.to_str()
            && let Some(first) = val.split(',').next()
        {
            let ip = first.trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
        // X-Real-IP
        if let Some(xri) = headers.get("x-real-ip")
            && let Ok(val) = xri.to_str()
        {
            let ip = val.trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    peer_ip.map(|ip| ip.to_string())
}

async fn register(
    State(state): State<Arc<AppState>>,
    conn_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let device = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .filter(|ua| ua.len() <= 512)
        .map(parse_device_name);
    let ip = extract_client_ip(&headers, Some(conn_info.0));

    let response = auth_service::register(&state, req, device.as_deref(), ip.as_deref()).await?;
    Ok(Json(response))
}

async fn login(
    State(state): State<Arc<AppState>>,
    conn_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let device = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .filter(|ua| ua.len() <= 512)
        .map(parse_device_name);
    let ip = extract_client_ip(&headers, Some(conn_info.0));

    let response = auth_service::login(&state, req, device.as_deref(), ip.as_deref()).await?;
    Ok(Json(response))
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    conn_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let device = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .filter(|ua| ua.len() <= 512)
        .map(parse_device_name);
    let ip = extract_client_ip(&headers, Some(conn_info.0));

    let response =
        auth_service::refresh_token(&state, req, device.as_deref(), ip.as_deref()).await?;
    Ok(Json(response))
}

/// Recover account using a recovery code (self-service password reset).
async fn recover_account(
    State(state): State<Arc<AppState>>,
    conn_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<RecoverAccountRequest>,
) -> Result<Json<RecoverAccountResponse>, AppError> {
    // Validate input lengths
    if req.username.len() > 32 || req.recovery_code.len() != 19 {
        return Err(AppError::Validation("invalid request".to_string()));
    }

    // Check account lockout (reuse login lockout to prevent brute-force)
    let ip = extract_client_ip(&headers, Some(conn_info.0));
    let lockout_key = format!("recover:{}", req.username);
    if let Some(remaining) = auth_service::check_lockout_by_key(&lockout_key) {
        return Err(AppError::Validation(format!(
            "too many attempts — try again in {remaining} seconds"
        )));
    }

    // Look up user
    let (user_id, recovery_hash) = match user_repo::get_user_for_recovery(&state.db, &req.username).await? {
        Some((id, Some(hash))) => (id, hash),
        _ => {
            // Record failed attempt but don't reveal whether user exists
            auth_service::record_failed_attempt(&lockout_key);
            return Err(AppError::Validation(
                "invalid username or recovery code".to_string(),
            ));
        }
    };

    // Verify recovery code (constant-time comparison to prevent timing attacks)
    let provided_hash = hex::encode(Sha256::digest(req.recovery_code.as_bytes()));
    if provided_hash.as_bytes().ct_eq(recovery_hash.as_bytes()).unwrap_u8() != 1 {
        auth_service::record_failed_attempt(&lockout_key);
        return Err(AppError::Validation(
            "invalid username or recovery code".to_string(),
        ));
    }

    // Validate new password
    auth_service::validate_password_public(&req.new_password)?;

    // Hash new password and update
    let new_hash = auth_service::hash_password_public(&req.new_password)?;
    user_repo::update_password(&state.db, user_id, &new_hash).await?;

    // Revoke all refresh tokens
    user_repo::revoke_all_refresh_tokens(&state.db, user_id).await?;

    // Generate new recovery code
    let (new_code, new_hash) = auth_service::generate_recovery_code();
    user_repo::set_recovery_code_hash(&state.db, user_id, &new_hash).await?;

    // Clear lockout on success
    auth_service::clear_lockout_by_key(&lockout_key);

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        uuid::Uuid::now_v7(),
        Some(user_id),
        "account_recovered",
        ip.as_deref(),
        None,
        None,
    )
    .await?;

    Ok(Json(RecoverAccountResponse {
        recovery_code: new_code,
    }))
}

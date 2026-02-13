use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};

use chatalot_common::api_types::{
    AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, ServerConfigResponse,
    TokenResponse,
};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::auth_service;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/config", get(server_config))
}

async fn server_config(
    State(state): State<Arc<AppState>>,
) -> Json<ServerConfigResponse> {
    Json(ServerConfigResponse {
        registration_mode: state.config.registration_mode.clone(),
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

/// Extract client IP from headers, checking reverse proxy headers first.
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // X-Forwarded-For (first entry is the client)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(val) = xff.to_str() {
            if let Some(first) = val.split(',').next() {
                let ip = first.trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }
    // X-Real-IP
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(val) = xri.to_str() {
            let ip = val.trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }
    None
}

async fn register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let device = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(parse_device_name);
    let ip = extract_client_ip(&headers);

    let response =
        auth_service::register(&state, req, device.as_deref(), ip.as_deref()).await?;
    Ok(Json(response))
}

async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let device = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(parse_device_name);
    let ip = extract_client_ip(&headers);

    let response =
        auth_service::login(&state, req, device.as_deref(), ip.as_deref()).await?;
    Ok(Json(response))
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let device = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(parse_device_name);
    let ip = extract_client_ip(&headers);

    let response =
        auth_service::refresh_token(&state, req, device.as_deref(), ip.as_deref()).await?;
    Ok(Json(response))
}

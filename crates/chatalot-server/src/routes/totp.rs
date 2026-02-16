use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;
use axum::{Extension, Json, Router};
use sha2::Digest;
use totp_rs::{Algorithm, Secret, TOTP};

use chatalot_common::api_types::{TotpSetupResponse, TotpVerifyRequest};
use chatalot_db::repos::user_repo;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/totp/setup", post(setup_totp))
        .route("/totp/verify", post(verify_totp))
        .route("/totp/disable", post(disable_totp))
}

/// Generate a TOTP secret and return the otpauth URI for QR code display.
/// The secret is stored encrypted but 2FA is not enabled until verify is called.
async fn setup_totp(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<TotpSetupResponse>, AppError> {
    let user = user_repo::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if user.totp_enabled {
        return Err(AppError::Conflict("2FA is already enabled".to_string()));
    }

    // Generate a random TOTP secret
    let secret = Secret::generate_secret();
    let secret_bytes = secret
        .to_bytes()
        .map_err(|e| AppError::Internal(format!("secret bytes: {e}")))?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes.clone(),
        Some("Chatalot".to_string()),
        user.username.clone(),
    )
    .map_err(|e| AppError::Internal(format!("totp init: {e}")))?;

    let otpauth_url = totp.get_url();
    let secret_base32 = secret.to_encoded().to_string();

    // Store the secret (not yet enabled — user must verify first)
    // Encrypt the secret if a TOTP encryption key is configured
    let stored_secret = encrypt_totp_secret(&secret_bytes, &state.config.totp_encryption_key);

    user_repo::set_totp_secret(&state.db, claims.sub, &stored_secret).await?;

    Ok(Json(TotpSetupResponse {
        otpauth_url,
        secret: secret_base32,
    }))
}

/// Verify a TOTP code to enable 2FA.
async fn verify_totp(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = user_repo::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let stored_secret = user.totp_secret.ok_or_else(|| {
        AppError::Validation("no TOTP secret configured, call /totp/setup first".to_string())
    })?;

    let secret_bytes = decrypt_totp_secret(&stored_secret, &state.config.totp_encryption_key);

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Chatalot".to_string()),
        user.username.clone(),
    )
    .map_err(|e| AppError::Internal(format!("totp init: {e}")))?;

    if !totp
        .check_current(&req.code)
        .map_err(|e| AppError::Internal(format!("totp check: {e}")))?
    {
        return Err(AppError::Validation("invalid TOTP code".to_string()));
    }

    // Enable TOTP
    user_repo::enable_totp(&state.db, claims.sub).await?;

    Ok(Json(serde_json::json!({ "enabled": true })))
}

/// Disable 2FA (requires a valid code).
async fn disable_totp(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = user_repo::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !user.totp_enabled {
        return Err(AppError::Validation("2FA is not enabled".to_string()));
    }

    let stored_secret = user
        .totp_secret
        .ok_or_else(|| AppError::Internal("totp_enabled but no secret".to_string()))?;

    let secret_bytes = decrypt_totp_secret(&stored_secret, &state.config.totp_encryption_key);

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Chatalot".to_string()),
        user.username.clone(),
    )
    .map_err(|e| AppError::Internal(format!("totp init: {e}")))?;

    if !totp
        .check_current(&req.code)
        .map_err(|e| AppError::Internal(format!("totp check: {e}")))?
    {
        return Err(AppError::Validation("invalid TOTP code".to_string()));
    }

    user_repo::disable_totp(&state.db, claims.sub).await?;

    Ok(Json(serde_json::json!({ "enabled": false })))
}

/// Verify a TOTP code during login (used by auth_service).
pub fn verify_totp_code(
    secret: &[u8],
    code: &str,
    encryption_key: &Option<String>,
) -> Result<bool, AppError> {
    let secret_bytes = decrypt_totp_secret(secret, encryption_key);

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes, None, String::new())
        .map_err(|e| AppError::Internal(format!("totp init: {e}")))?;

    totp.check_current(code)
        .map_err(|e| AppError::Internal(format!("totp check: {e}")))
}

// Simple XOR-based encryption for TOTP secret at rest.
// In production, use AES-GCM with the TOTP_ENCRYPTION_KEY.
fn encrypt_totp_secret(secret: &[u8], key: &Option<String>) -> Vec<u8> {
    match key {
        Some(k) => {
            let key_bytes = sha2::Sha256::digest(k.as_bytes());
            secret
                .iter()
                .enumerate()
                .map(|(i, b)| b ^ key_bytes[i % 32])
                .collect()
        }
        None => secret.to_vec(),
    }
}

fn decrypt_totp_secret(encrypted: &[u8], key: &Option<String>) -> Vec<u8> {
    // XOR is symmetric
    encrypt_totp_secret(encrypted, key)
}

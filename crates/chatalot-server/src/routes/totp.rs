use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;
use axum::{Extension, Json, Router};
use sha2::Digest;
use totp_rs::{Algorithm, Secret, TOTP};

use chatalot_common::api_types::{BackupCodesResponse, TotpEnableResponse, TotpSetupResponse, TotpVerifyRequest};
use chatalot_db::repos::user_repo;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::services::auth_service;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/totp/setup", post(setup_totp))
        .route("/totp/verify", post(verify_totp))
        .route("/totp/disable", post(disable_totp))
        .route(
            "/totp/regenerate-backup-codes",
            post(regenerate_backup_codes),
        )
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

/// Verify a TOTP code to enable 2FA. Returns backup codes on success.
async fn verify_totp(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<TotpEnableResponse>, AppError> {
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

    // Generate backup codes
    let (codes, hashes) = auth_service::generate_backup_codes(8);
    user_repo::set_totp_backup_codes(&state.db, claims.sub, &hashes).await?;

    Ok(Json(TotpEnableResponse {
        enabled: true,
        backup_codes: codes,
    }))
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

    // Clear backup codes since TOTP is disabled
    user_repo::set_totp_backup_codes(&state.db, claims.sub, &[]).await?;

    Ok(Json(serde_json::json!({ "enabled": false })))
}

/// Regenerate backup codes (requires a valid TOTP code).
async fn regenerate_backup_codes(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<BackupCodesResponse>, AppError> {
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

    // Generate new backup codes (replaces old ones)
    let (codes, hashes) = auth_service::generate_backup_codes(8);
    user_repo::set_totp_backup_codes(&state.db, claims.sub, &hashes).await?;

    Ok(Json(BackupCodesResponse {
        backup_codes: codes,
    }))
}

/// Verify a TOTP code during login (used by auth_service).
pub fn verify_totp_code(
    secret: &[u8],
    code: &str,
    encryption_key: &str,
) -> Result<bool, AppError> {
    let secret_bytes = decrypt_totp_secret(secret, encryption_key);

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes, None, String::new())
        .map_err(|e| AppError::Internal(format!("totp init: {e}")))?;

    totp.check_current(code)
        .map_err(|e| AppError::Internal(format!("totp check: {e}")))
}

/// AEAD overhead: 12-byte nonce + 16-byte Poly1305 tag = 28 bytes.
const AEAD_OVERHEAD: usize = 12 + 16;

/// Encrypt a TOTP secret using ChaCha20-Poly1305.
/// Output format: nonce (12 bytes) || ciphertext+tag.
fn encrypt_totp_secret(secret: &[u8], key: &str) -> Vec<u8> {
    use chacha20poly1305::{ChaCha20Poly1305, KeyInit, AeadCore, aead::Aead};
    let key_bytes = sha2::Sha256::digest(key.as_bytes());
    let cipher = ChaCha20Poly1305::new((&key_bytes[..]).into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut rand::rngs::OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, secret)
        .expect("TOTP secret encryption failed");
    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    out
}

/// Decrypt a TOTP secret. Transparently handles both:
/// - New format: ChaCha20-Poly1305 (len > plaintext + 28 bytes overhead)
/// - Legacy format: XOR with SHA-256 of key (len == plaintext, no nonce/tag)
fn decrypt_totp_secret(encrypted: &[u8], key: &str) -> Vec<u8> {
    let key_bytes = sha2::Sha256::digest(key.as_bytes());

    // TOTP secrets are typically 20 bytes (SHA1) or 32 bytes (SHA256).
    // AEAD adds 28 bytes overhead, so AEAD-encrypted data is always >= 28 bytes
    // and the plaintext would be (len - 28). Legacy XOR has no overhead.
    // Detect: if len > 28 and decryption succeeds, it's AEAD; otherwise try legacy XOR.
    if encrypted.len() > AEAD_OVERHEAD {
        use chacha20poly1305::{ChaCha20Poly1305, KeyInit, aead::Aead};
        let cipher = ChaCha20Poly1305::new((&key_bytes[..]).into());
        let nonce = chacha20poly1305::Nonce::from_slice(&encrypted[..12]);
        if let Ok(plaintext) = cipher.decrypt(nonce, &encrypted[12..]) {
            return plaintext;
        }
        // AEAD decryption failed — fall through to legacy XOR
    }

    // Legacy XOR decryption (for secrets encrypted before the ChaCha20 upgrade)
    encrypted
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key_bytes[i % 32])
        .collect()
}

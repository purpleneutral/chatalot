use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use sha2::{Digest, Sha256};

use chatalot_common::api_types::{
    KeyBundleResponse, KeyRegistrationRequest, OneTimePrekeyResponse, OneTimePrekeyUpload,
    SignedPrekeyResponse, SignedPrekeyUpload,
};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::repos::key_repo;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

const KEYS_LOW_THRESHOLD: i64 = 25;
const MAX_OTP_BATCH_SIZE: usize = 200;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/keys/{user_id}/bundle", get(get_key_bundle))
        .route("/keys/register", post(register_keys))
        .route("/keys/prekeys/signed", post(upload_signed_prekey))
        .route("/keys/prekeys/one-time", post(upload_one_time_prekeys))
        .route("/keys/prekeys/count", get(get_prekey_count))
}

/// Fetch a user's key bundle for X3DH session setup.
async fn get_key_bundle(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<KeyBundleResponse>, AppError> {
    let bundle = key_repo::fetch_key_bundle(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("key bundle not found".to_string()))?;

    // Warn the user if their one-time prekeys are running low
    if let Ok(remaining) = key_repo::count_unused_prekeys(&state.db, user_id).await
        && remaining < KEYS_LOW_THRESHOLD
    {
        state.connections.send_to_user(
            &user_id,
            &ServerMessage::KeysLow {
                remaining: remaining as u32,
            },
        );
    }

    Ok(Json(KeyBundleResponse {
        identity_key: bundle.identity_key,
        signed_prekey: SignedPrekeyResponse {
            key_id: bundle.signed_prekey.key_id,
            public_key: bundle.signed_prekey.public_key,
            signature: bundle.signed_prekey.signature,
        },
        one_time_prekey: bundle.one_time_prekey.map(|otpk| OneTimePrekeyResponse {
            key_id: otpk.key_id,
            public_key: otpk.public_key,
        }),
    }))
}

/// Upload or rotate a signed prekey.
async fn upload_signed_prekey(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<SignedPrekeyUpload>,
) -> Result<(), AppError> {
    if req.public_key.len() != 32 {
        return Err(AppError::Validation(
            "public key must be 32 bytes".to_string(),
        ));
    }
    if req.signature.len() != 64 {
        return Err(AppError::Validation(
            "signature must be 64 bytes".to_string(),
        ));
    }

    key_repo::upsert_signed_prekey(
        &state.db,
        Uuid::now_v7(),
        claims.sub,
        req.key_id,
        &req.public_key,
        &req.signature,
    )
    .await?;
    Ok(())
}

/// Upload a batch of one-time prekeys.
async fn upload_one_time_prekeys(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(prekeys): Json<Vec<OneTimePrekeyUpload>>,
) -> Result<(), AppError> {
    if prekeys.len() > MAX_OTP_BATCH_SIZE {
        return Err(AppError::Validation(
            format!("maximum {MAX_OTP_BATCH_SIZE} one-time prekeys per upload"),
        ));
    }

    let pairs: Vec<(i32, Vec<u8>)> = prekeys
        .into_iter()
        .map(|p| (p.key_id, p.public_key))
        .collect();

    key_repo::upload_one_time_prekeys(&state.db, claims.sub, &pairs).await?;
    Ok(())
}

/// Get the count of remaining unused one-time prekeys.
async fn get_prekey_count(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = key_repo::count_unused_prekeys(&state.db, claims.sub).await?;
    Ok(Json(serde_json::json!({ "count": count })))
}

/// Register all E2E keys for users who registered before E2E was active.
async fn register_keys(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<KeyRegistrationRequest>,
) -> Result<(), AppError> {
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
            "signature must be 64 bytes".to_string(),
        ));
    }
    if req.one_time_prekeys.len() > MAX_OTP_BATCH_SIZE {
        return Err(AppError::Validation(
            format!("maximum {MAX_OTP_BATCH_SIZE} one-time prekeys per upload"),
        ));
    }

    let fingerprint = hex::encode(Sha256::digest(&req.identity_key));

    // Upsert identity key
    key_repo::upsert_identity_key(&state.db, claims.sub, &req.identity_key, &fingerprint).await?;

    // Upsert signed prekey
    key_repo::upsert_signed_prekey(
        &state.db,
        Uuid::now_v7(),
        claims.sub,
        req.signed_prekey.key_id,
        &req.signed_prekey.public_key,
        &req.signed_prekey.signature,
    )
    .await?;

    // Upload one-time prekeys
    if !req.one_time_prekeys.is_empty() {
        let pairs: Vec<(i32, Vec<u8>)> = req
            .one_time_prekeys
            .into_iter()
            .map(|p| (p.key_id, p.public_key))
            .collect();
        key_repo::upload_one_time_prekeys(&state.db, claims.sub, &pairs).await?;
    }

    tracing::info!(user_id = %claims.sub, "Late E2E key registration completed");
    Ok(())
}

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};

use chatalot_common::api_types::{PushSubscribeRequest, PushUnsubscribeRequest, VapidKeyResponse};
use chatalot_db::repos::push_subscription_repo;

use crate::app_state::AppState;
use crate::middleware::auth::AccessClaims;

/// Public route: returns the VAPID public key so clients can subscribe.
async fn get_vapid_key(
    State(state): State<Arc<AppState>>,
) -> Result<Json<VapidKeyResponse>, StatusCode> {
    let key = state
        .config
        .vapid_public_key
        .as_ref()
        .filter(|k| !k.is_empty())
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(VapidKeyResponse {
        public_key: key.clone(),
    }))
}

/// Subscribe the client's push endpoint.
async fn subscribe(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<PushSubscribeRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Validate inputs
    if req.endpoint.len() > 2048 {
        return Err((StatusCode::BAD_REQUEST, "Endpoint URL too long".into()));
    }
    if req.p256dh_key.is_empty() || req.auth_key.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing encryption keys".into()));
    }
    if !req.endpoint.starts_with("https://") {
        return Err((StatusCode::BAD_REQUEST, "Endpoint must use HTTPS".into()));
    }

    push_subscription_repo::upsert_subscription(
        &state.db,
        uuid::Uuid::new_v4(),
        claims.sub,
        &req.endpoint,
        &req.p256dh_key,
        &req.auth_key,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to save push subscription: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save subscription".into(),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Unsubscribe a push endpoint.
async fn unsubscribe(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<PushUnsubscribeRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    push_subscription_repo::delete_by_endpoint(&state.db, claims.sub, &req.endpoint)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete push subscription: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to remove subscription".into(),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Public routes (no auth required) — VAPID key endpoint.
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new().route("/push/vapid-key", get(get_vapid_key))
}

/// Protected routes (auth required) — subscribe/unsubscribe.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/push/subscribe", post(subscribe))
        .route("/push/unsubscribe", post(unsubscribe))
}

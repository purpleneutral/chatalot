use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{SenderKeyDistributionResponse, UploadSenderKeyRequest};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::repos::{channel_repo, sender_key_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/channels/{id}/sender-keys", post(upload_sender_key))
        .route("/channels/{id}/sender-keys", get(get_sender_keys))
}

/// Upload (or rotate) this user's sender key distribution for a channel.
async fn upload_sender_key(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<UploadSenderKeyRequest>,
) -> Result<Json<SenderKeyDistributionResponse>, AppError> {
    // Verify membership
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let row = sender_key_repo::upsert_distribution(
        &state.db,
        Uuid::now_v7(),
        channel_id,
        claims.sub,
        req.chain_id,
        &req.distribution,
    )
    .await?;

    // Notify channel members about the new/rotated sender key
    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::SenderKeyUpdated {
            channel_id,
            user_id: claims.sub,
            chain_id: req.chain_id,
            distribution: req.distribution,
        },
    );

    Ok(Json(SenderKeyDistributionResponse {
        id: row.id,
        channel_id: row.channel_id,
        user_id: row.user_id,
        chain_id: row.chain_id,
        distribution: row.distribution,
        created_at: row.created_at.to_rfc3339(),
    }))
}

/// Fetch all sender key distributions for a channel.
async fn get_sender_keys(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<SenderKeyDistributionResponse>>, AppError> {
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let rows = sender_key_repo::get_channel_distributions(&state.db, channel_id).await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| SenderKeyDistributionResponse {
                id: r.id,
                channel_id: r.channel_id,
                user_id: r.user_id,
                chain_id: r.chain_id,
                distribution: r.distribution,
                created_at: r.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

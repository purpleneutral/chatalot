use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{MessageResponse, MessagesQuery, SearchQuery};
use chatalot_db::repos::{channel_repo, message_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/channels/{id}/messages", get(get_messages))
        .route("/channels/{id}/messages/search", get(search_messages))
}

async fn get_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    // Verify membership
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let messages = message_repo::get_messages(&state.db, channel_id, query.before, limit).await?;

    let responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|m| MessageResponse {
            id: m.id,
            channel_id: m.channel_id,
            sender_id: m.sender_id,
            ciphertext: m.ciphertext,
            nonce: m.nonce,
            message_type: m.message_type,
            reply_to_id: m.reply_to_id,
            sender_key_id: m.sender_key_id,
            edited_at: m.edited_at.map(|t| t.to_rfc3339()),
            created_at: m.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

async fn search_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let limit = query.limit.unwrap_or(20).min(50);
    let messages =
        message_repo::search_messages(&state.db, channel_id, &query.q, limit).await?;

    let responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|m| MessageResponse {
            id: m.id,
            channel_id: m.channel_id,
            sender_id: m.sender_id,
            ciphertext: m.ciphertext,
            nonce: m.nonce,
            message_type: m.message_type,
            reply_to_id: m.reply_to_id,
            sender_key_id: m.sender_key_id,
            edited_at: m.edited_at.map(|t| t.to_rfc3339()),
            created_at: m.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

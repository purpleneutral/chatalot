use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    MessageResponse, MessagesQuery, PinnedMessageResponse, SearchQuery,
};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::repos::{channel_repo, message_repo, pin_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::permissions;

const MAX_PINS_PER_CHANNEL: i64 = 50;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/channels/{id}/messages", get(get_messages))
        .route("/channels/{id}/messages/search", get(search_messages))
        .route("/channels/{id}/pins", get(list_pins))
        .route(
            "/channels/{id}/pins/{msg_id}",
            post(pin_message).delete(unpin_message),
        )
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

// ── Pinned Messages ──

async fn list_pins(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<PinnedMessageResponse>>, AppError> {
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let pins = pin_repo::list_pinned_messages(&state.db, channel_id).await?;
    let responses: Vec<PinnedMessageResponse> = pins
        .into_iter()
        .map(|p| PinnedMessageResponse {
            id: p.id,
            channel_id: p.channel_id,
            sender_id: p.sender_id,
            ciphertext: p.ciphertext,
            nonce: p.nonce,
            message_type: p.message_type,
            reply_to_id: p.reply_to_id,
            sender_key_id: p.sender_key_id,
            edited_at: p.edited_at.map(|t| t.to_rfc3339()),
            created_at: p.created_at.to_rfc3339(),
            pinned_by: p.pinned_by,
            pinned_at: p.pinned_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

async fn pin_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, msg_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    // Check channel membership and role
    let role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_delete_others_messages(&role) {
        return Err(AppError::Forbidden);
    }

    // Verify the message belongs to this channel and is not deleted
    let msg = message_repo::get_message_by_id(&state.db, msg_id)
        .await?
        .ok_or_else(|| AppError::NotFound("message not found".to_string()))?;

    if msg.channel_id != channel_id {
        return Err(AppError::Validation(
            "message does not belong to this channel".to_string(),
        ));
    }

    if msg.deleted_at.is_some() {
        return Err(AppError::Validation(
            "cannot pin a deleted message".to_string(),
        ));
    }

    // Check pin limit
    let count = pin_repo::count_pins(&state.db, channel_id).await?;
    if count >= MAX_PINS_PER_CHANNEL {
        return Err(AppError::Validation(format!(
            "maximum of {MAX_PINS_PER_CHANNEL} pins per channel"
        )));
    }

    // Check if already pinned
    if pin_repo::is_pinned(&state.db, msg_id).await? {
        return Err(AppError::Conflict("message is already pinned".to_string()));
    }

    let pin = pin_repo::pin_message(&state.db, msg_id, channel_id, claims.sub).await?;

    // Broadcast to channel subscribers
    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::MessagePinned {
            message_id: msg_id,
            channel_id,
            pinned_by: claims.sub,
            pinned_at: pin.pinned_at.to_rfc3339(),
        },
    );

    Ok(())
}

async fn unpin_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, msg_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    let role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_delete_others_messages(&role) {
        return Err(AppError::Forbidden);
    }

    if !pin_repo::unpin_message(&state.db, msg_id).await? {
        return Err(AppError::NotFound("pin not found".to_string()));
    }

    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::MessageUnpinned {
            message_id: msg_id,
            channel_id,
        },
    );

    Ok(())
}

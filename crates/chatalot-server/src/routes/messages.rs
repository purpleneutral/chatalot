use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    MessageEditResponse, MessageResponse, MessagesQuery, PinnedMessageResponse, ReactionInfo,
    SearchQuery,
};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::repos::{channel_repo, message_repo, pin_repo, reaction_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::permissions;

use message_repo::SearchFilters;

const DEFAULT_MAX_PINS: i64 = 50;

fn build_search_filters(query: &SearchQuery) -> SearchFilters {
    SearchFilters {
        sender: query.sender.clone(),
        before: query
            .before
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::Utc)),
        after: query
            .after
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::Utc)),
        has_file: query.has_file,
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/channels/{id}/messages", get(get_messages))
        .route("/channels/{id}/messages/search", get(search_messages))
        .route(
            "/channels/{id}/messages/{msg_id}/history",
            get(get_edit_history),
        )
        .route(
            "/channels/{id}/threads/{msg_id}",
            get(get_thread_messages),
        )
        .route("/messages/search", get(global_search_messages))
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
    let root_only = query.root_only.unwrap_or(false);
    let messages = message_repo::get_messages(&state.db, channel_id, query.before, limit, root_only).await?;

    let reactions_map = fetch_reactions_map(&state.db, &messages).await?;
    let thread_map = fetch_thread_map(&state.db, &messages).await?;
    Ok(Json(messages_to_responses(messages, reactions_map, thread_map)))
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

    let q_len = query.q.chars().count();
    if !(2..=256).contains(&q_len) {
        return Err(AppError::Validation(
            "search query must be 2-256 characters".to_string(),
        ));
    }

    let limit = query.limit.unwrap_or(20).min(50);
    let filters = build_search_filters(&query);
    let messages =
        message_repo::search_messages(&state.db, channel_id, &query.q, limit, &filters).await?;

    let reactions_map = fetch_reactions_map(&state.db, &messages).await?;
    let thread_map = fetch_thread_map(&state.db, &messages).await?;
    Ok(Json(messages_to_responses(messages, reactions_map, thread_map)))
}

async fn global_search_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    let q_len = query.q.chars().count();
    if !(2..=256).contains(&q_len) {
        return Err(AppError::Validation(
            "search query must be 2-256 characters".to_string(),
        ));
    }

    let limit = query.limit.unwrap_or(20).min(50);
    let filters = build_search_filters(&query);
    let messages =
        message_repo::search_messages_global(&state.db, claims.sub, &query.q, limit, &filters)
            .await?;

    let reactions_map = fetch_reactions_map(&state.db, &messages).await?;
    let thread_map = fetch_thread_map(&state.db, &messages).await?;
    Ok(Json(messages_to_responses(messages, reactions_map, thread_map)))
}

// ── Threads ──

async fn get_thread_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, msg_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    // Verify the root message belongs to this channel
    let root = message_repo::get_message_by_id(&state.db, msg_id)
        .await?
        .ok_or_else(|| AppError::NotFound("thread root not found".into()))?;
    if root.channel_id != channel_id {
        return Err(AppError::NotFound("thread root not found".into()));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let messages = message_repo::get_thread_messages(&state.db, msg_id, query.before, limit).await?;

    let reactions_map = fetch_reactions_map(&state.db, &messages).await?;
    let empty_thread_map = std::collections::HashMap::new();
    Ok(Json(messages_to_responses(messages, reactions_map, empty_thread_map)))
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
    // Check channel membership and role (instance owner/admin bypass)
    let channel_role = channel_repo::get_member_role(&state.db, channel_id, claims.sub).await?;
    let role = permissions::effective_role(channel_role.as_deref(), claims.is_owner, claims.is_admin);

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
    let max_pins = state.instance_settings.read().await.max_pins_per_channel;
    let count = pin_repo::count_pins(&state.db, channel_id).await?;
    if count >= max_pins {
        return Err(AppError::Validation(format!(
            "maximum of {max_pins} pins per channel"
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
    let channel_role = channel_repo::get_member_role(&state.db, channel_id, claims.sub).await?;
    let role = permissions::effective_role(channel_role.as_deref(), claims.is_owner, claims.is_admin);

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

// ── Helpers ──

async fn fetch_reactions_map(
    db: &sqlx::PgPool,
    messages: &[chatalot_db::models::message::Message],
) -> Result<std::collections::HashMap<Uuid, Vec<ReactionInfo>>, AppError> {
    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let reaction_rows = reaction_repo::get_reactions_for_messages(db, &message_ids).await?;
    let mut map: std::collections::HashMap<Uuid, Vec<ReactionInfo>> =
        std::collections::HashMap::new();
    for r in reaction_rows {
        map.entry(r.message_id)
            .or_default()
            .push(ReactionInfo {
                emoji: r.emoji,
                user_ids: r.user_ids,
            });
    }
    Ok(map)
}

async fn fetch_thread_map(
    db: &sqlx::PgPool,
    messages: &[chatalot_db::models::message::Message],
) -> Result<std::collections::HashMap<Uuid, message_repo::ThreadInfo>, AppError> {
    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let thread_infos = message_repo::get_thread_reply_counts(db, &message_ids).await?;
    Ok(thread_infos
        .into_iter()
        .map(|t| (t.root_message_id, t))
        .collect())
}

fn messages_to_responses(
    messages: Vec<chatalot_db::models::message::Message>,
    mut reactions_map: std::collections::HashMap<Uuid, Vec<ReactionInfo>>,
    thread_map: std::collections::HashMap<Uuid, message_repo::ThreadInfo>,
) -> Vec<MessageResponse> {
    messages
        .into_iter()
        .map(|m| {
            let reactions = reactions_map.remove(&m.id).unwrap_or_default();
            let thread_info = thread_map.get(&m.id);
            MessageResponse {
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
                reactions,
                thread_id: m.thread_id,
                thread_reply_count: thread_info.map(|t| t.reply_count),
                thread_last_reply_at: thread_info.map(|t| t.last_reply_at.to_rfc3339()),
            }
        })
        .collect()
}

// ── Edit History ──

async fn get_edit_history(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((channel_id, msg_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<MessageEditResponse>>, AppError> {
    // Verify membership
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    // Verify message belongs to channel
    let msg = message_repo::get_message_by_id(&state.db, msg_id)
        .await?
        .ok_or_else(|| AppError::NotFound("message not found".into()))?;
    if msg.channel_id != channel_id {
        return Err(AppError::NotFound("message not found".into()));
    }

    let edits = message_repo::get_edit_history(&state.db, msg_id).await?;
    Ok(Json(
        edits
            .into_iter()
            .map(|e| MessageEditResponse {
                id: e.id,
                old_ciphertext: e.old_ciphertext,
                old_nonce: e.old_nonce,
                edited_at: e.edited_at.to_rfc3339(),
            })
            .collect(),
    ))
}

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{patch, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    CreateWebhookRequest, ExecuteWebhookRequest, UpdateWebhookRequest, WebhookResponse,
};
use chatalot_common::ws_messages::{MessageType, ServerMessage};
use chatalot_db::repos::{channel_repo, message_repo, webhook_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::permissions;

/// Protected routes (require auth).
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/channels/{id}/webhooks", post(create_webhook).get(list_webhooks))
        .route("/webhooks/{id}", patch(update_webhook).delete(delete_webhook))
}

/// Public route for executing webhooks (no auth — uses token).
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new().route("/webhooks/{token}", post(execute_webhook))
}

async fn create_webhook(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<WebhookResponse>, AppError> {
    // Must be admin/owner of channel
    let role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_manage_roles(&role) {
        return Err(AppError::Forbidden);
    }

    if req.name.is_empty() || req.name.len() > 64 {
        return Err(AppError::Validation("webhook name must be 1-64 characters".into()));
    }

    let id = Uuid::now_v7();
    let token = generate_token();

    let webhook = webhook_repo::create(
        &state.db,
        id,
        channel_id,
        &req.name,
        &token,
        claims.sub,
        req.avatar_url.as_deref(),
    )
    .await?;

    Ok(Json(webhook_to_response(&webhook)))
}

async fn list_webhooks(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<WebhookResponse>>, AppError> {
    let role = channel_repo::get_member_role(&state.db, channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_manage_roles(&role) {
        return Err(AppError::Forbidden);
    }

    let webhooks = webhook_repo::list_for_channel(&state.db, channel_id).await?;
    Ok(Json(webhooks.iter().map(webhook_to_response).collect()))
}

async fn update_webhook(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(webhook_id): Path<Uuid>,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, AppError> {
    let webhook = webhook_repo::get_by_id(&state.db, webhook_id)
        .await?
        .ok_or_else(|| AppError::NotFound("webhook not found".into()))?;

    let role = channel_repo::get_member_role(&state.db, webhook.channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_manage_roles(&role) {
        return Err(AppError::Forbidden);
    }

    let updated = webhook_repo::update(
        &state.db,
        webhook_id,
        req.name.as_deref(),
        req.avatar_url.as_ref().map(|v| v.as_deref()),
        req.active,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("webhook not found".into()))?;

    Ok(Json(webhook_to_response(&updated)))
}

async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(webhook_id): Path<Uuid>,
) -> Result<(), AppError> {
    let webhook = webhook_repo::get_by_id(&state.db, webhook_id)
        .await?
        .ok_or_else(|| AppError::NotFound("webhook not found".into()))?;

    let role = channel_repo::get_member_role(&state.db, webhook.channel_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    if !permissions::can_manage_roles(&role) {
        return Err(AppError::Forbidden);
    }

    webhook_repo::delete(&state.db, webhook_id).await?;
    Ok(())
}

async fn execute_webhook(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
    Json(req): Json<ExecuteWebhookRequest>,
) -> Result<(), AppError> {
    let webhook = webhook_repo::get_by_token(&state.db, &token)
        .await?
        .ok_or_else(|| AppError::NotFound("webhook not found or inactive".into()))?;

    if req.content.is_empty() || req.content.len() > 4000 {
        return Err(AppError::Validation("content must be 1-4000 characters".into()));
    }

    let message_id = Uuid::now_v7();
    let display_name = req.username.as_deref().unwrap_or(&webhook.name);
    let plaintext = serde_json::json!({
        "content": req.content,
        "webhook_name": display_name,
        "webhook_avatar": req.avatar_url.as_deref().or(webhook.avatar_url.as_deref()),
    })
    .to_string();

    let stored = message_repo::create_webhook_message(
        &state.db,
        message_id,
        webhook.channel_id,
        &plaintext,
    )
    .await?;

    // Broadcast to channel
    state.connections.broadcast_to_channel(
        webhook.channel_id,
        ServerMessage::NewMessage {
            id: message_id,
            channel_id: webhook.channel_id,
            sender_id: webhook.created_by,
            ciphertext: vec![0],
            nonce: vec![0],
            message_type: MessageType::Webhook,
            reply_to: None,
            sender_key_id: None,
            created_at: stored.created_at.to_rfc3339(),
        },
    );

    Ok(())
}

fn webhook_to_response(w: &chatalot_db::models::webhook::Webhook) -> WebhookResponse {
    WebhookResponse {
        id: w.id,
        channel_id: w.channel_id,
        name: w.name.clone(),
        token: w.token.clone(),
        avatar_url: w.avatar_url.clone(),
        active: w.active,
        created_at: w.created_at.to_rfc3339(),
    }
}

fn generate_token() -> String {
    use rand::Rng as _;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

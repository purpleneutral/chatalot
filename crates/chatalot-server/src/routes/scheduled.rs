use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{ScheduleMessageRequest, ScheduledMessageResponse};
use chatalot_db::repos::{channel_repo, scheduled_message_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

const MAX_SCHEDULE_DAYS_AHEAD: i64 = 30;
const MAX_SCHEDULED_PER_USER: usize = 50;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/messages/schedule", post(schedule_message))
        .route("/messages/scheduled", get(list_scheduled))
        .route("/messages/scheduled/{id}", delete(cancel_scheduled))
}

async fn schedule_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<ScheduleMessageRequest>,
) -> Result<Json<ScheduledMessageResponse>, AppError> {
    // Verify channel membership
    if !channel_repo::is_member(&state.db, req.channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let scheduled_for = chrono::DateTime::parse_from_rfc3339(&req.scheduled_for)
        .map_err(|_| AppError::Validation("invalid scheduled_for datetime".into()))?
        .with_timezone(&chrono::Utc);

    if scheduled_for <= chrono::Utc::now() {
        return Err(AppError::Validation(
            "scheduled_for must be in the future".into(),
        ));
    }

    // Limit how far in the future a message can be scheduled
    if scheduled_for > chrono::Utc::now() + chrono::TimeDelta::days(MAX_SCHEDULE_DAYS_AHEAD) {
        return Err(AppError::Validation(
            format!("cannot schedule more than {MAX_SCHEDULE_DAYS_AHEAD} days ahead"),
        ));
    }

    if req.ciphertext.is_empty() || req.nonce.is_empty() {
        return Err(AppError::Validation(
            "ciphertext and nonce are required".into(),
        ));
    }

    // Cap sizes to prevent abuse (100 KB ciphertext, 256 bytes nonce)
    if req.ciphertext.len() > 100_000 {
        return Err(AppError::Validation(
            "ciphertext too large (max 100 KB)".into(),
        ));
    }
    if req.nonce.len() > 256 {
        return Err(AppError::Validation("nonce too large (max 256 bytes)".into()));
    }

    // Enforce per-user limit
    let existing = scheduled_message_repo::list_for_user(&state.db, claims.sub).await?;
    if existing.len() >= MAX_SCHEDULED_PER_USER {
        return Err(AppError::Validation(
            format!("cannot have more than {MAX_SCHEDULED_PER_USER} scheduled messages"),
        ));
    }

    let id = Uuid::now_v7();
    let msg = scheduled_message_repo::create(
        &state.db,
        id,
        req.channel_id,
        claims.sub,
        &req.ciphertext,
        &req.nonce,
        scheduled_for,
    )
    .await?;

    Ok(Json(ScheduledMessageResponse {
        id: msg.id,
        channel_id: msg.channel_id,
        scheduled_for: msg.scheduled_for.to_rfc3339(),
        created_at: msg.created_at.to_rfc3339(),
    }))
}

async fn list_scheduled(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<ScheduledMessageResponse>>, AppError> {
    let messages = scheduled_message_repo::list_for_user(&state.db, claims.sub).await?;
    Ok(Json(
        messages
            .iter()
            .map(|m| ScheduledMessageResponse {
                id: m.id,
                channel_id: m.channel_id,
                scheduled_for: m.scheduled_for.to_rfc3339(),
                created_at: m.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn cancel_scheduled(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    if !scheduled_message_repo::delete(&state.db, id, claims.sub).await? {
        return Err(AppError::NotFound("scheduled message not found".into()));
    }
    Ok(())
}

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::AnnouncementResponse;
use chatalot_db::repos::announcement_repo;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/announcements", get(list_undismissed))
        .route("/announcements/{id}/dismiss", post(dismiss))
}

async fn list_undismissed(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<AnnouncementResponse>>, AppError> {
    let announcements = announcement_repo::list_undismissed(&state.db, claims.sub).await?;
    Ok(Json(
        announcements
            .iter()
            .map(|a| AnnouncementResponse {
                id: a.id,
                title: a.title.clone(),
                body: a.body.clone(),
                created_by: a.created_by,
                created_at: a.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn dismiss(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(announcement_id): Path<Uuid>,
) -> Result<(), AppError> {
    announcement_repo::dismiss(&state.db, claims.sub, announcement_id).await?;
    Ok(())
}

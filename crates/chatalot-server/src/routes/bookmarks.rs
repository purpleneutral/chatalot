use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{delete, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{BookmarkResponse, CreateBookmarkRequest, PaginationQuery};
use chatalot_db::repos::{bookmark_repo, channel_repo, message_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/bookmarks", post(add_bookmark).get(list_bookmarks))
        .route("/bookmarks/{id}", delete(remove_bookmark))
}

async fn add_bookmark(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateBookmarkRequest>,
) -> Result<Json<BookmarkResponse>, AppError> {
    if let Some(ref note) = req.note
        && note.len() > 500
    {
        return Err(AppError::Validation(
            "note must be at most 500 characters".into(),
        ));
    }

    // Verify the user is a member of the channel containing this message
    let msg = message_repo::get_message_by_id(&state.db, req.message_id)
        .await?
        .ok_or_else(|| AppError::NotFound("message not found".to_string()))?;
    if !channel_repo::is_member(&state.db, msg.channel_id, claims.sub)
        .await
        .unwrap_or(false)
    {
        return Err(AppError::Forbidden);
    }

    let id = Uuid::now_v7();
    let bookmark = bookmark_repo::create(
        &state.db,
        id,
        claims.sub,
        req.message_id,
        req.note.as_deref(),
    )
    .await?;

    Ok(Json(BookmarkResponse {
        id: bookmark.id,
        message_id: bookmark.message_id,
        note: bookmark.note,
        created_at: bookmark.created_at.to_rfc3339(),
    }))
}

async fn list_bookmarks(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<BookmarkResponse>>, AppError> {
    let limit = pagination.limit.unwrap_or(100).clamp(1, 500);
    let offset = pagination.offset.unwrap_or(0).max(0);
    let bookmarks = bookmark_repo::list_for_user(&state.db, claims.sub, limit, offset).await?;
    Ok(Json(
        bookmarks
            .iter()
            .map(|b| BookmarkResponse {
                id: b.id,
                message_id: b.message_id,
                note: b.note.clone(),
                created_at: b.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn remove_bookmark(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    if !bookmark_repo::delete(&state.db, id, claims.sub).await? {
        return Err(AppError::NotFound("bookmark not found".into()));
    }
    Ok(())
}

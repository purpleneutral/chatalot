use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Extension, Json, Router};

use chatalot_common::api_types::{UserPublic, UserSearchQuery};
use chatalot_db::repos::user_repo;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/users/search", get(search_users))
}

async fn search_users(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<AccessClaims>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<Vec<UserPublic>>, AppError> {
    if query.q.len() < 2 {
        return Err(AppError::Validation(
            "search query must be at least 2 characters".to_string(),
        ));
    }

    let users = user_repo::search_users(&state.db, &query.q, 20).await?;
    let results = users
        .iter()
        .map(|u| UserPublic {
            id: u.id,
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            avatar_url: u.avatar_url.clone(),
            status: u.status.clone(),
            custom_status: u.custom_status.clone(),
            is_admin: u.is_admin,
        })
        .collect();

    Ok(Json(results))
}

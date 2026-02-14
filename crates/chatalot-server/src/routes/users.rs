use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{UserPublic, UserSearchQuery};
use chatalot_db::repos::{community_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/search", get(search_users))
        .route("/users/{user_id}", get(get_user))
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPublic>, AppError> {
    let user = user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    // Only visible if caller shares a community with the target (or is admin)
    if !claims.is_admin
        && claims.sub != user_id
        && !community_repo::shares_community(&state.db, claims.sub, user_id).await?
    {
        return Err(AppError::NotFound("user not found".to_string()));
    }

    Ok(Json(UserPublic {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        status: user.status,
        custom_status: user.custom_status,
        is_admin: user.is_admin,
        created_at: Some(user.created_at.to_rfc3339()),
    }))
}

async fn search_users(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<Vec<UserPublic>>, AppError> {
    if query.q.len() < 2 {
        return Err(AppError::Validation(
            "search query must be at least 2 characters".to_string(),
        ));
    }

    // Instance admins can search all users; everyone else only sees community peers
    let users = if claims.is_admin {
        user_repo::search_users(&state.db, &query.q, 20).await?
    } else {
        community_repo::search_visible_users(&state.db, claims.sub, &query.q, 20).await?
    };

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
            created_at: Some(u.created_at.to_rfc3339()),
        })
        .collect();

    Ok(Json(results))
}

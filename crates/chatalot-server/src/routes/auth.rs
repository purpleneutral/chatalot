use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use chatalot_common::api_types::{
    AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, ServerConfigResponse,
    TokenResponse,
};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::auth_service;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/config", get(server_config))
}

async fn server_config(
    State(state): State<Arc<AppState>>,
) -> Json<ServerConfigResponse> {
    Json(ServerConfigResponse {
        registration_mode: state.config.registration_mode.clone(),
    })
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = auth_service::register(&state, req).await?;
    Ok(Json(response))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = auth_service::login(&state, req).await?;
    Ok(Json(response))
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let response = auth_service::refresh_token(&state, req).await?;
    Ok(Json(response))
}

use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use chatalot_common::api_types::HealthResponse;

use crate::app_state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime,
    })
}

pub mod account;
pub mod admin;
pub mod auth;
pub mod channels;
pub mod communities;
pub mod dms;
pub mod feedback;
pub mod gifs;
pub mod groups;
pub mod files;
pub mod health;
pub mod link_preview;
pub mod keys;
pub mod messages;
pub mod totp;
pub mod users;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::app_state::AppState;
use crate::middleware::auth::auth_middleware;
use crate::middleware::community_gate::community_gate_middleware;
use crate::middleware::rate_limit::{auth_rate_limit_middleware, rate_limit_middleware};
use crate::middleware::security::security_headers;
use crate::ws::session::ws_upgrade;

pub fn build_router(state: Arc<AppState>) -> Router {
    // Auth routes with stricter rate limiting
    let auth_routes = Router::new()
        .merge(auth::routes())
        .layer(axum::middleware::from_fn(auth_rate_limit_middleware));

    // Public routes (no auth required)
    let public_routes = auth_routes
        .merge(health::routes())
        .merge(account::public_routes());

    // Community-gated routes (require auth + community membership)
    let community_gated_routes = Router::new()
        .merge(communities::gated_routes())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            community_gate_middleware,
        ));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .merge(channels::routes())
        .merge(groups::routes())
        .merge(messages::routes())
        .merge(keys::routes())
        .merge(dms::routes())
        .merge(files::routes())
        .merge(totp::routes())
        .merge(users::routes())
        .merge(feedback::routes())
        .merge(gifs::routes())
        .merge(link_preview::routes())
        .merge(account::routes())
        .merge(admin::routes())
        .merge(communities::public_routes())
        .merge(community_gated_routes)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Static file serving for the SPA (Svelte build output)
    let static_dir = std::env::var("STATIC_FILES_PATH").unwrap_or_else(|_| "./static".to_string());
    let spa_fallback = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(format!("{static_dir}/index.html")));

    // CORS is permissive because the desktop client (Tauri) makes cross-origin
    // requests from a local file:// origin. All endpoints require JWT auth,
    // so CORS is not the access gate.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api", public_routes.merge(protected_routes))
        .route("/ws", get(ws_upgrade))
        .fallback_service(spa_fallback)
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

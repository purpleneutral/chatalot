pub mod account;
pub mod admin;
pub mod announcements;
pub mod auth;
pub mod bookmarks;
pub mod channels;
pub mod communities;
pub mod dms;
pub mod feedback;
pub mod files;
pub mod gifs;
pub mod groups;
pub mod health;
pub mod keys;
pub mod legal;
pub mod link_preview;
pub mod messages;
pub mod polls;
pub mod push;
pub mod scheduled;
pub mod sender_keys;
pub mod totp;
pub mod users;
pub mod webhooks;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::header;
use axum::routing::get;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use axum::http::StatusCode;
use tower_http::set_header::SetResponseHeaderLayer;
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
        .merge(legal::routes())
        .merge(account::public_routes())
        .merge(webhooks::public_routes())
        .merge(push::public_routes());

    // Community-gated routes (require auth + community membership)
    let community_gated_routes = Router::new().merge(communities::gated_routes()).layer(
        axum::middleware::from_fn_with_state(state.clone(), community_gate_middleware),
    );

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .merge(channels::routes())
        .merge(groups::routes())
        .merge(messages::routes())
        .merge(keys::routes())
        .merge(sender_keys::routes())
        .merge(dms::routes())
        .merge(files::routes())
        .merge(totp::routes())
        .merge(users::routes())
        .merge(feedback::routes())
        .merge(gifs::routes())
        .merge(link_preview::routes())
        .merge(account::routes())
        .merge(admin::routes())
        .merge(webhooks::routes())
        .merge(polls::routes())
        .merge(scheduled::routes())
        .merge(bookmarks::routes())
        .merge(announcements::routes())
        .merge(push::routes())
        .merge(communities::public_routes())
        .merge(community_gated_routes)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Static file serving for the SPA (Svelte build output)
    let static_dir = std::env::var("STATIC_FILES_PATH").unwrap_or_else(|_| "./static".to_string());

    // Service worker must never be cached by browsers or CDNs
    let sw_service = ServeFile::new(format!("{static_dir}/sw.js"));
    let sw_route = Router::new().route_service("/sw.js", sw_service).layer(
        SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-store"),
        ),
    );

    // Serve favicon.png at /favicon.ico to prevent Chrome's automatic 404
    let favicon_route = Router::new().route_service(
        "/favicon.ico",
        ServeFile::new(format!("{static_dir}/favicon.png")),
    );

    // SPA fallback: serve static files, or index.html for client-side routes.
    // We use a custom service_fn instead of ServeFile because ServeDir's
    // not_found_service preserves a 404 status code even when content is served.
    let index_html_path = format!("{static_dir}/index.html");
    let spa_fallback = ServeDir::new(&static_dir).fallback(tower::service_fn(
        move |_req: axum::http::Request<axum::body::Body>| {
            let path = index_html_path.clone();
            async move {
                let bytes = tokio::fs::read(&path).await.unwrap_or_default();
                Ok::<_, std::convert::Infallible>(
                    axum::http::Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .body(axum::body::Body::from(bytes))
                        .unwrap(),
                )
            }
        },
    ));

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
        .merge(sw_route)
        .merge(favicon_route)
        .fallback_service(spa_fallback)
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(cors)
        .layer(DefaultBodyLimit::max(110 * 1024 * 1024)) // 110MB (slightly above MAX_FILE_SIZE_MB default)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

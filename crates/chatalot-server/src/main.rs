mod app_state;
mod config;
mod error;
mod middleware;
pub mod permissions;
mod routes;
mod services;
mod ws;

use std::sync::Arc;
use std::time::Instant;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app_state::AppState;
use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "chatalot_server=debug,tower_http=debug".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting Chatalot server on {}", config.listen_addr);

    // Create database pool and run migrations
    let db_pool = chatalot_db::pool::create_pool(&config.database_url).await?;
    tracing::info!("Database connected");

    chatalot_db::pool::run_migrations(&db_pool).await?;
    tracing::info!("Migrations applied");

    // Build application state
    let start_time = Instant::now();
    let state = Arc::new(AppState::new(config.clone(), db_pool, start_time)?);

    // Seed admin user from env var if configured
    if let Some(ref admin_username) = config.admin_username {
        match chatalot_db::repos::user_repo::ensure_admin(&state.db, admin_username).await {
            Ok(true) => tracing::info!("Granted admin to user '{admin_username}'"),
            Ok(false) => tracing::debug!("User '{admin_username}' is already admin or does not exist"),
            Err(e) => tracing::warn!("Failed to seed admin user '{admin_username}': {e}"),
        }
    }

    // Build the router
    let app = routes::build_router(state.clone());

    // Start the server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("Listening on {}", config.listen_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received");
}

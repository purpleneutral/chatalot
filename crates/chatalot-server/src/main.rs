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
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chatalot_server=debug,tower_http=debug".into()),
        )
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
            Ok(false) => {
                tracing::debug!("User '{admin_username}' is already admin or does not exist")
            }
            Err(e) => tracing::warn!("Failed to seed admin user '{admin_username}': {e}"),
        }
    }

    // Spawn background task: typing indicator timeout (10s)
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                let expired = state
                    .connections
                    .expire_typing(std::time::Duration::from_secs(10));
                for (channel_id, user_id) in expired {
                    state.connections.broadcast_to_channel(
                        channel_id,
                        chatalot_common::ws_messages::ServerMessage::UserStoppedTyping {
                            channel_id,
                            user_id,
                        },
                    );
                }
            }
        });
    }

    // Spawn background task: broadcast channel cleanup (every 5 minutes)
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                let removed = state.connections.cleanup_idle_channels();
                if removed > 0 {
                    tracing::debug!("Cleaned up {removed} idle broadcast channels");
                }
            }
        });
    }

    // Spawn background task: periodic data cleanup (every hour)
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            // Wait 1 minute after startup before first cleanup
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                // Delete refresh tokens expired more than 7 days ago
                match sqlx::query(
                    "DELETE FROM refresh_tokens WHERE expires_at < NOW() - INTERVAL '7 days'",
                )
                .execute(&db)
                .await
                {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            tracing::info!(
                                "Cleaned up {} expired refresh tokens",
                                r.rows_affected()
                            );
                        }
                    }
                    Err(e) => tracing::warn!("Failed to clean expired tokens: {e}"),
                }
                // Delete used one-time prekeys older than 30 days
                match sqlx::query("DELETE FROM one_time_prekeys WHERE used = true AND created_at < NOW() - INTERVAL '30 days'")
                    .execute(&db)
                    .await
                {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            tracing::info!("Cleaned up {} used prekeys", r.rows_affected());
                        }
                    }
                    Err(e) => tracing::warn!("Failed to clean used prekeys: {e}"),
                }
                // Prune audit logs older than 90 days
                match sqlx::query(
                    "DELETE FROM audit_log WHERE created_at < NOW() - INTERVAL '90 days'",
                )
                .execute(&db)
                .await
                {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            tracing::info!(
                                "Cleaned up {} audit log entries older than 90 days",
                                r.rows_affected()
                            );
                        }
                    }
                    Err(e) => tracing::warn!("Failed to clean old audit logs: {e}"),
                }
                // End orphaned voice sessions (no participants, still active)
                match sqlx::query(
                    "UPDATE voice_sessions SET ended_at = NOW() WHERE ended_at IS NULL AND id NOT IN (SELECT DISTINCT session_id FROM voice_session_participants WHERE left_at IS NULL)"
                )
                    .execute(&db)
                    .await
                {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            tracing::info!("Cleaned up {} orphaned voice sessions", r.rows_affected());
                        }
                    }
                    Err(e) => tracing::warn!("Failed to clean orphaned voice sessions: {e}"),
                }
            }
        });
    }

    // Spawn background task: soft-delete message GC (daily, 5min startup delay)
    // Hard-deletes messages that were soft-deleted more than 30 days ago
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(24 * 60 * 60));
            loop {
                interval.tick().await;
                match chatalot_db::repos::message_repo::gc_soft_deleted(&db, 30).await {
                    Ok(0) => {}
                    Ok(n) => {
                        tracing::info!("GC: hard-deleted {n} messages soft-deleted >30 days ago")
                    }
                    Err(e) => tracing::warn!("GC soft-delete cleanup failed: {e}"),
                }
            }
        });
    }

    // Spawn background task: orphan file cleanup (daily, 2h startup delay)
    // Removes disk files with no DB record and DB records with missing disk files
    {
        let db = state.db.clone();
        let storage_path = state.config.file_storage_path.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(7200)).await;
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(24 * 60 * 60));
            loop {
                interval.tick().await;

                // Get all known file paths from DB
                let db_files = match chatalot_db::repos::file_repo::list_all_file_paths(&db).await {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::warn!("Orphan cleanup: failed to list DB files: {e}");
                        continue;
                    }
                };

                let db_paths: std::collections::HashSet<String> =
                    db_files.iter().map(|(_, p)| p.clone()).collect();

                // Walk storage directory and find disk files with no DB record
                let mut orphan_disk_files = 0u64;
                let storage = std::path::Path::new(&storage_path);
                if let Ok(mut shard_dirs) = tokio::fs::read_dir(storage).await {
                    while let Ok(Some(shard_entry)) = shard_dirs.next_entry().await {
                        let shard_path = shard_entry.path();
                        if !shard_path.is_dir() {
                            continue;
                        }
                        if let Ok(mut files) = tokio::fs::read_dir(&shard_path).await {
                            while let Ok(Some(file_entry)) = files.next_entry().await {
                                let file_path = file_entry.path();
                                let path_str = file_path.to_string_lossy().to_string();
                                if !db_paths.contains(&path_str) {
                                    if let Err(e) = tokio::fs::remove_file(&file_path).await {
                                        tracing::warn!(
                                            "Orphan cleanup: failed to remove {path_str}: {e}"
                                        );
                                    } else {
                                        orphan_disk_files += 1;
                                    }
                                }
                            }
                        }
                    }
                }

                if orphan_disk_files > 0 {
                    tracing::info!(
                        "Orphan cleanup: removed {orphan_disk_files} disk files with no DB record"
                    );
                }
            }
        });
    }

    // Spawn background task: scheduled message delivery (every 30s)
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                match chatalot_db::repos::scheduled_message_repo::get_due_messages(&state.db).await
                {
                    Ok(messages) => {
                        for msg in messages {
                            // Verify user is still a member before delivering
                            match chatalot_db::repos::channel_repo::is_member(
                                &state.db,
                                msg.channel_id,
                                msg.user_id,
                            )
                            .await
                            {
                                Ok(false) | Err(_) => {
                                    tracing::info!(
                                        "Dropping scheduled message {}: user no longer a member",
                                        msg.id
                                    );
                                    let _ =
                                        chatalot_db::repos::scheduled_message_repo::delete_by_id(
                                            &state.db, msg.id,
                                        )
                                        .await;
                                    continue;
                                }
                                Ok(true) => {}
                            }

                            // Deliver the message as if the user sent it now
                            let message_id = uuid::Uuid::now_v7();
                            match chatalot_db::repos::message_repo::create_message(
                                &state.db,
                                message_id,
                                msg.channel_id,
                                msg.user_id,
                                msg.ciphertext.as_bytes(),
                                msg.nonce.as_bytes(),
                                "text",
                                None,
                                None,
                                None,
                                None,
                            )
                            .await
                            {
                                Ok(stored) => {
                                    state.connections.broadcast_to_channel(
                                        msg.channel_id,
                                        chatalot_common::ws_messages::ServerMessage::NewMessage {
                                            id: message_id,
                                            channel_id: msg.channel_id,
                                            sender_id: msg.user_id,
                                            ciphertext: msg.ciphertext.into_bytes(),
                                            nonce: msg.nonce.into_bytes(),
                                            message_type:
                                                chatalot_common::ws_messages::MessageType::Text,
                                            reply_to: None,
                                            sender_key_id: None,
                                            created_at: stored.created_at.to_rfc3339(),
                                        },
                                    );
                                    // Delete the scheduled message after delivery
                                    let _ =
                                        chatalot_db::repos::scheduled_message_repo::delete_by_id(
                                            &state.db, msg.id,
                                        )
                                        .await;
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to deliver scheduled message {}: {e}",
                                        msg.id
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to query due scheduled messages: {e}");
                    }
                }
            }
        });
    }

    // Spawn background task: message expiry cleanup (every 5 min)
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                match chatalot_db::repos::message_repo::delete_expired_messages(&db).await {
                    Ok(0) => {}
                    Ok(n) => tracing::info!("Expired {n} messages past their TTL"),
                    Err(e) => tracing::warn!("Failed to delete expired messages: {e}"),
                }
            }
        });
    }

    // Spawn background task: timeout cleanup (every 5 min)
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                let _ = chatalot_db::repos::timeout_repo::cleanup_expired(&db).await;
            }
        });
    }

    // Spawn background task: in-memory cache cleanup (every 10 min)
    {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600));
            loop {
                interval.tick().await;
                let gifs = routes::gifs::cleanup_gif_cache();
                let previews = routes::link_preview::cleanup_preview_cache();
                if gifs > 0 || previews > 0 {
                    tracing::debug!(
                        "Cache cleanup: evicted {gifs} GIF entries, {previews} link preview entries"
                    );
                }
            }
        });
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

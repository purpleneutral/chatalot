use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

use crate::config::Config;
use crate::services::push_service::PushService;
use crate::ws::connection_manager::ConnectionManager;

/// Admin-configurable instance settings (loaded from DB, cached in memory).
#[derive(Debug, Clone)]
pub struct InstanceSettings {
    pub max_messages_cache: u32,
    pub max_pins_per_channel: i64,
    pub e2e_enabled: bool,
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            max_messages_cache: 500,
            max_pins_per_channel: 50,
            e2e_enabled: true,
        }
    }
}

pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
    pub start_time: Instant,
    pub connections: ConnectionManager,
    pub client_version: String,
    pub http_client: reqwest::Client,
    /// In-memory set of suspended user IDs for instant JWT rejection.
    pub suspended_users: dashmap::DashSet<uuid::Uuid>,
    /// Web Push notification service (None if VAPID keys not configured).
    pub push_service: Option<Arc<PushService>>,
    /// Admin-configurable instance settings (cached in memory).
    pub instance_settings: tokio::sync::RwLock<InstanceSettings>,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, start_time: Instant) -> Result<Self> {
        let private_pem = std::fs::read_to_string(&config.jwt_private_key_path)?;
        let public_pem = std::fs::read_to_string(&config.jwt_public_key_path)?;

        let jwt_encoding_key = EncodingKey::from_ed_pem(private_pem.as_bytes())?;
        let jwt_decoding_key = DecodingKey::from_ed_pem(public_pem.as_bytes())?;

        // Read client version from static/version.json (written by Vite build)
        let static_dir =
            std::env::var("STATIC_FILES_PATH").unwrap_or_else(|_| "./static".to_string());
        let client_version = std::fs::read_to_string(format!("{static_dir}/version.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["version"].as_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()?;

        // Initialize push service if VAPID keys are configured
        let push_service = config
            .vapid_private_key
            .as_ref()
            .and_then(|key| {
                if key.is_empty() {
                    return None;
                }
                match PushService::new(key, config.public_url.as_deref()) {
                    Ok(svc) => {
                        tracing::info!("Web Push notifications enabled");
                        Some(Arc::new(svc))
                    }
                    Err(e) => {
                        tracing::warn!("Web Push disabled: {e}");
                        None
                    }
                }
            });

        Ok(Self {
            config,
            db,
            jwt_encoding_key,
            jwt_decoding_key,
            start_time,
            connections: ConnectionManager::new(),
            client_version,
            http_client,
            suspended_users: dashmap::DashSet::new(),
            push_service,
            instance_settings: tokio::sync::RwLock::new(InstanceSettings::default()),
        })
    }
}

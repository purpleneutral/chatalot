use std::time::Instant;

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

use crate::config::Config;
use crate::ws::connection_manager::ConnectionManager;

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
        })
    }
}

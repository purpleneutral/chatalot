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
}

impl AppState {
    pub fn new(config: Config, db: PgPool, start_time: Instant) -> Result<Self> {
        let private_pem = std::fs::read_to_string(&config.jwt_private_key_path)?;
        let public_pem = std::fs::read_to_string(&config.jwt_public_key_path)?;

        let jwt_encoding_key = EncodingKey::from_ed_pem(private_pem.as_bytes())?;
        let jwt_decoding_key = DecodingKey::from_ed_pem(public_pem.as_bytes())?;

        Ok(Self {
            config,
            db,
            jwt_encoding_key,
            jwt_decoding_key,
            start_time,
            connections: ConnectionManager::new(),
        })
    }
}

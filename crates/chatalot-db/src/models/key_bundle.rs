use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SignedPrekey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_id: i32,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OneTimePrekey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_id: i32,
    pub public_key: Vec<u8>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

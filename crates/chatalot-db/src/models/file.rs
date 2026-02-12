use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileRecord {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub encrypted_name: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub storage_path: String,
    pub checksum: String,
    pub channel_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DmPair {
    pub user_a: Uuid,
    pub user_b: Uuid,
    pub channel_id: Uuid,
    pub created_at: DateTime<Utc>,
}

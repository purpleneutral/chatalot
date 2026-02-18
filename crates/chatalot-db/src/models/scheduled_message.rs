use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub ciphertext: String,
    pub nonce: String,
    pub content_preview: Option<String>,
    pub scheduled_for: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

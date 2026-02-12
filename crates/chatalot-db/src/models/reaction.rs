use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reaction {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadCursor {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub last_read_message_id: Option<Uuid>,
    pub last_read_at: DateTime<Utc>,
}

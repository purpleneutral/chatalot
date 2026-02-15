use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Poll {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub created_by: Uuid,
    pub question: String,
    pub options: serde_json::Value,
    pub multi_select: bool,
    pub anonymous: bool,
    pub closed: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PollVote {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub user_id: Uuid,
    pub option_index: i32,
    pub created_at: DateTime<Utc>,
}

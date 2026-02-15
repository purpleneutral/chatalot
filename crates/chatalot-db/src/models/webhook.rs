use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub name: String,
    pub token: String,
    pub created_by: Uuid,
    pub avatar_url: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

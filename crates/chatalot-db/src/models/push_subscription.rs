use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub failure_count: i32,
}

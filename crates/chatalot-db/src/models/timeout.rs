use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserTimeout {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub issued_by: Uuid,
    pub reason: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

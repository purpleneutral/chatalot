use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockedHash {
    pub id: Uuid,
    pub hash: String,
    pub reason: Option<String>,
    pub blocked_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

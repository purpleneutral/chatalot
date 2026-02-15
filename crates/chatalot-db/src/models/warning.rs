use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Warning {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub issued_by: Uuid,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

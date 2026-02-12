use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct RegistrationInvite {
    pub id: Uuid,
    pub code: String,
    pub created_by: Uuid,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

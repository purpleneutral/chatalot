use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct Report {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub report_type: String,
    pub target_id: Uuid,
    pub reason: String,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub admin_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

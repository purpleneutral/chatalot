use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomEmoji {
    pub id: Uuid,
    pub community_id: Uuid,
    pub shortcode: String,
    pub file_path: String,
    pub content_type: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub message_type: String,
    pub sender_key_id: Option<Uuid>,
    pub reply_to_id: Option<Uuid>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub plaintext: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub quarantined_at: Option<DateTime<Utc>>,
    pub quarantined_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

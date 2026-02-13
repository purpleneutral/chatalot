use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PinnedMessage {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    pub pinned_by: Uuid,
    pub pinned_at: DateTime<Utc>,
}

/// Flat struct joining message fields + pin metadata for the list query.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PinnedMessageWithContent {
    // Message fields
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
    pub created_at: DateTime<Utc>,
    // Pin fields
    pub pinned_by: Uuid,
    pub pinned_at: DateTime<Utc>,
}

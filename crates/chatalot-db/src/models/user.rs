use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub voice_background_url: Option<String>,
    pub status: String,
    pub custom_status: Option<String>,
    pub totp_secret: Option<Vec<u8>>,
    pub totp_enabled: bool,
    pub is_admin: bool,
    pub is_owner: bool,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspended_reason: Option<String>,
    pub recovery_code_hash: Option<String>,
    pub totp_backup_codes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IdentityKey {
    pub user_id: Uuid,
    pub identity_key: Vec<u8>,
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: Vec<u8>,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

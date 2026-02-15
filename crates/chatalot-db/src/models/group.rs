use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub community_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub visibility: String,
    pub discoverable: bool,
    pub assigned_member_id: Option<Uuid>,
    pub allow_invites: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupMemberInfo {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupInvite {
    pub id: Uuid,
    pub group_id: Uuid,
    pub created_by: Uuid,
    pub code: String,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

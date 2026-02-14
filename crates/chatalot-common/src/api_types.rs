use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Auth ──

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
    /// Ed25519 public identity key (32 bytes)
    pub identity_key: Vec<u8>,
    /// Signed prekey bundle
    pub signed_prekey: SignedPrekeyUpload,
    /// Initial batch of one-time prekeys
    pub one_time_prekeys: Vec<OneTimePrekeyUpload>,
    /// Invite code (required when server is in invite_only mode)
    #[serde(default)]
    pub invite_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedPrekeyUpload {
    pub key_id: i32,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneTimePrekeyUpload {
    pub key_id: i32,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// ── Users ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub custom_status: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

// ── Keys ──

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyBundleResponse {
    pub identity_key: Vec<u8>,
    pub signed_prekey: SignedPrekeyResponse,
    pub one_time_prekey: Option<OneTimePrekeyResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedPrekeyResponse {
    pub key_id: i32,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneTimePrekeyResponse {
    pub key_id: i32,
    pub public_key: Vec<u8>,
}

// ── Channels ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: String,
    pub topic: Option<String>,
    pub group_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub channel_type: String,
    pub topic: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: String,
    pub group_id: Option<Uuid>,
}

// ── Channel Members ──

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelMemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BanRequest {
    pub reason: Option<String>,
}

// ── Messages ──

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub message_type: String,
    pub reply_to_id: Option<Uuid>,
    pub sender_key_id: Option<Uuid>,
    pub edited_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagesQuery {
    /// Cursor for pagination (message UUID; fetch messages before this)
    pub before: Option<Uuid>,
    /// Number of messages to return (max 100)
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

// ── DMs ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDmRequest {
    pub target_user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmChannelResponse {
    pub channel: ChannelResponse,
    pub other_user: UserPublic,
}

// ── Files ──

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub id: Uuid,
    pub size_bytes: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadataResponse {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub encrypted_name: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub checksum: String,
    pub created_at: String,
}

// ── TOTP 2FA ──

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    /// otpauth:// URI for QR code generation
    pub otpauth_url: String,
    /// Base32-encoded secret for manual entry
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpVerifyRequest {
    pub code: String,
}

// ── Users ──

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSearchQuery {
    pub q: String,
}

// ── Feedback ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedbackRequest {
    pub title: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackResponse {
    pub success: bool,
    pub issue_number: Option<u64>,
    pub message: String,
}

// ── Groups ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub community_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub community_id: Uuid,
    pub created_at: String,
    pub member_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
}

// ── Invites ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_in_hours: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteResponse {
    pub id: Uuid,
    pub code: String,
    pub group_id: Uuid,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteInfoResponse {
    pub group_name: String,
    pub group_description: Option<String>,
    pub member_count: i64,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInviteResponse {
    pub group_id: Uuid,
    pub group_name: String,
}

// ── Link Preview ──

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkPreviewQuery {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkPreviewResponse {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub site_name: Option<String>,
}

// ── Account Management ──

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub custom_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutAllResponse {
    pub revoked_count: u64,
}

// ── Ownership Transfer ──

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferOwnershipRequest {
    pub new_owner_id: Uuid,
}

// ── Admin ──

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
    pub suspended_at: Option<String>,
    pub suspended_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuspendUserRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetAdminRequest {
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUsersQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ── Registration Invites (server-wide) ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRegistrationInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationInviteResponse {
    pub id: Uuid,
    pub code: String,
    pub created_by: Uuid,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub expires_at: Option<String>,
    pub created_at: String,
}

// ── Server Config (public) ──

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigResponse {
    pub registration_mode: String,
}

// ── Health ──

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

// ── Communities ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommunityRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCommunityRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub owner_id: Uuid,
    pub created_at: String,
    pub member_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityMemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub nickname: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetCommunityRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetNicknameRequest {
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityBanRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommunityInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_in_hours: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityInviteResponse {
    pub id: Uuid,
    pub code: String,
    pub community_id: Uuid,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityInviteInfoResponse {
    pub community_name: String,
    pub community_description: Option<String>,
    pub member_count: i64,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptCommunityInviteResponse {
    pub community_id: Uuid,
    pub community_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferCommunityOwnershipRequest {
    pub new_owner_id: Uuid,
}

// ── Pinned Messages ──

#[derive(Debug, Serialize, Deserialize)]
pub struct PinnedMessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub message_type: String,
    pub reply_to_id: Option<Uuid>,
    pub sender_key_id: Option<Uuid>,
    pub edited_at: Option<String>,
    pub created_at: String,
    pub pinned_by: Uuid,
    pub pinned_at: String,
}

// ── User Preferences ──

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferencesResponse {
    pub preferences: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub preferences: serde_json::Value,
}

// ── GIFs ──

#[derive(Debug, Serialize, Deserialize)]
pub struct GifSearchQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
    pub pos: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifResult {
    pub id: String,
    pub title: String,
    pub preview_url: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GifSearchResponse {
    pub results: Vec<GifResult>,
    pub next: Option<String>,
}

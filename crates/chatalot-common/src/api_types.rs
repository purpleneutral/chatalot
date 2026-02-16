use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_true() -> bool {
    true
}

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
    /// Recovery code shown once at registration (not included on login)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecoverAccountRequest {
    pub username: String,
    pub recovery_code: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecoverAccountResponse {
    /// New recovery code (the previous one is invalidated)
    pub recovery_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegenerateRecoveryCodeResponse {
    pub recovery_code: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    pub status: String,
    pub custom_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub is_owner: bool,
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
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub slow_mode_seconds: i32,
    #[serde(default = "default_true")]
    pub discoverable: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_background: Option<String>,
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
pub struct ReactionInfo {
    pub emoji: String,
    pub user_ids: Vec<Uuid>,
}

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<ReactionInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_reply_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_last_reply_at: Option<String>,
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
    pub sender: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub has_file: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageEditResponse {
    pub id: Uuid,
    pub old_ciphertext: Vec<u8>,
    pub old_nonce: Vec<u8>,
    pub edited_at: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpEnableResponse {
    pub enabled: bool,
    /// One-time backup codes for account recovery if TOTP device is lost
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupCodesResponse {
    pub backup_codes: Vec<String>,
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
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub assigned_member_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub discoverable: Option<bool>,
    pub allow_invites: Option<bool>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub accent_color: Option<String>,
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
    pub visibility: String,
    pub discoverable: bool,
    pub assigned_member_id: Option<Uuid>,
    pub allow_invites: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
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
    pub read_only: Option<bool>,
    pub slow_mode_seconds: Option<i32>,
    pub message_ttl_seconds: Option<i32>,
    pub discoverable: Option<bool>,
    pub archived: Option<bool>,
    pub voice_background: Option<String>,
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
    pub banner_url: Option<String>,
    pub custom_status: Option<String>,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
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
    #[serde(default)]
    pub groups: Vec<AdminUserMembership>,
    #[serde(default)]
    pub communities: Vec<AdminUserMembership>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminUserMembership {
    pub id: Uuid,
    pub name: String,
    pub role: String,
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

/// Reusable pagination query parameters for list endpoints.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ── Admin Security Suite ──

#[derive(Debug, Serialize, Deserialize)]
pub struct PurgeParams {
    #[serde(default)]
    pub block_hashes: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurgeResult {
    pub messages_deleted: u64,
    pub files_deleted: u64,
    pub hashes_blocked: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminFilesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub user_id: Option<Uuid>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminFileEntry {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub encrypted_name: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub checksum: String,
    pub channel_id: Option<Uuid>,
    pub quarantined_at: Option<String>,
    pub quarantined_by: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminFilesResponse {
    pub files: Vec<AdminFileEntry>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageStatsResponse {
    pub total_files: i64,
    pub total_bytes: i64,
    pub per_user: Vec<UserStorageStatResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStorageStatResponse {
    pub user_id: Uuid,
    pub file_count: i64,
    pub total_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddBlockedHashRequest {
    pub hash: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockedHashResponse {
    pub id: Uuid,
    pub hash: String,
    pub reason: Option<String>,
    pub blocked_by: Uuid,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub action: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditLogEntryResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogEntryResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_url: Option<String>,
}

// ── Health ──

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub db_healthy: bool,
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
    pub who_can_create_groups: Option<String>,
    pub who_can_create_invites: Option<String>,
    pub discoverable: Option<bool>,
    pub banner_url: Option<String>,
    pub community_theme: Option<serde_json::Value>,
    pub welcome_message: Option<String>,
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
    pub who_can_create_groups: String,
    pub who_can_create_invites: String,
    pub discoverable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub community_theme: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub welcome_message: Option<String>,
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

// ── Sender Keys ──

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadSenderKeyRequest {
    pub chain_id: i32,
    pub distribution: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SenderKeyDistributionResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub chain_id: i32,
    pub distribution: serde_json::Value,
    pub created_at: String,
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

// ── User Blocking ──

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockUserRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockedUserResponse {
    pub blocked_id: Uuid,
    pub created_at: String,
}

// ── Content Reports ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReportRequest {
    pub report_type: String,
    pub target_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportResponse {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub report_type: String,
    pub target_id: Uuid,
    pub reason: String,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<String>,
    pub admin_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportsQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportsResponse {
    pub reports: Vec<ReportResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewReportRequest {
    pub status: String,
    pub admin_notes: Option<String>,
}

// ── Webhooks ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWebhookRequest {
    pub name: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub name: String,
    /// Only set when webhook is first created; omitted in list responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub avatar_url: Option<String>,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteWebhookRequest {
    pub content: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

// ── Timeouts ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTimeoutRequest {
    pub user_id: Uuid,
    pub duration_seconds: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeoutResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub issued_by: Uuid,
    pub reason: Option<String>,
    pub expires_at: String,
    pub created_at: String,
}

// ── Warnings ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWarningRequest {
    pub user_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WarningResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub issued_by: Uuid,
    pub reason: String,
    pub created_at: String,
}

// ── Scheduled Messages ──

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleMessageRequest {
    pub channel_id: Uuid,
    pub ciphertext: String,
    pub nonce: String,
    pub scheduled_for: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduledMessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub scheduled_for: String,
    pub created_at: String,
}

// ── Polls ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePollRequest {
    pub question: String,
    pub options: Vec<String>,
    #[serde(default)]
    pub multi_select: bool,
    #[serde(default)]
    pub anonymous: bool,
    pub expires_in_minutes: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PollResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub created_by: Uuid,
    pub question: String,
    pub options: Vec<String>,
    pub multi_select: bool,
    pub anonymous: bool,
    pub closed: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub votes: Vec<PollOptionVotes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PollOptionVotes {
    pub option_index: i32,
    pub count: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voter_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VotePollRequest {
    pub option_index: i32,
}

// ── Bookmarks ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBookmarkRequest {
    pub message_id: Uuid,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookmarkResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub note: Option<String>,
    pub created_at: String,
}

// ── Custom Emoji ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomEmojiResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub shortcode: String,
    pub url: String,
    pub content_type: String,
    pub uploaded_by: Uuid,
    pub created_at: String,
}

// ── Announcements ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnouncementResponse {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub created_by: Uuid,
    pub created_at: String,
}

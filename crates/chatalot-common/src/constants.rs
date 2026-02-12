/// Access token lifetime in seconds (15 minutes)
pub const ACCESS_TOKEN_LIFETIME_SECS: i64 = 900;

/// Refresh token lifetime in seconds (30 days)
pub const REFRESH_TOKEN_LIFETIME_SECS: i64 = 30 * 24 * 60 * 60;

/// Maximum username length
pub const MAX_USERNAME_LEN: usize = 32;

/// Maximum display name length
pub const MAX_DISPLAY_NAME_LEN: usize = 64;

/// Maximum message size in bytes (64 KiB ciphertext)
pub const MAX_MESSAGE_SIZE: usize = 65_536;

/// Maximum file size in bytes (default 100 MiB)
pub const DEFAULT_MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Number of one-time prekeys to upload initially
pub const INITIAL_ONE_TIME_PREKEYS: u32 = 100;

/// Warn client when prekeys fall below this threshold
pub const PREKEY_LOW_THRESHOLD: u32 = 20;

/// WebSocket heartbeat interval in seconds
pub const WS_HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Presence offline grace period in seconds
pub const PRESENCE_OFFLINE_GRACE_SECS: u64 = 30;

/// Typing indicator timeout in seconds
pub const TYPING_TIMEOUT_SECS: u64 = 5;

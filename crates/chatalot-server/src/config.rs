use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub jwt_private_key_path: String,
    pub jwt_public_key_path: String,
    pub totp_encryption_key: String,
    pub file_storage_path: String,
    pub max_file_size_mb: u64,
    pub github_api_token: Option<String>,
    pub github_repo_owner: Option<String>,
    pub github_repo_name: Option<String>,
    pub admin_username: Option<String>,
    pub registration_mode: String,
    pub community_creation_mode: String,
    pub public_url: Option<String>,
    /// ICE servers for WebRTC (JSON array of {urls, username?, credential?}).
    pub ice_servers_json: Option<String>,
    /// VAPID private key for Web Push (base64 URL-safe encoded).
    pub vapid_private_key: Option<String>,
    /// VAPID public key for Web Push (base64 URL-safe encoded).
    pub vapid_public_key: Option<String>,
    /// Per-user upload quota in MB (0 = unlimited). Default 500 MB.
    pub upload_quota_mb: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let max_file_size_mb = std::env::var("MAX_FILE_SIZE_MB")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100u64)
            .clamp(1, 10_000);

        let registration_mode = std::env::var("REGISTRATION_MODE")
            .unwrap_or_else(|_| "invite_only".to_string());
        let registration_mode = match registration_mode.as_str() {
            "open" | "invite_only" | "closed" => registration_mode,
            other => {
                tracing::warn!(
                    "Invalid REGISTRATION_MODE '{other}', falling back to 'invite_only'"
                );
                "invite_only".to_string()
            }
        };

        let community_creation_mode = std::env::var("COMMUNITY_CREATION_MODE")
            .unwrap_or_else(|_| "admin_only".to_string());
        let community_creation_mode = match community_creation_mode.as_str() {
            "open" | "admin_only" => community_creation_mode,
            other => {
                tracing::warn!(
                    "Invalid COMMUNITY_CREATION_MODE '{other}', falling back to 'admin_only'"
                );
                "admin_only".to_string()
            }
        };

        let ice_servers_json = std::env::var("ICE_SERVERS").ok();
        if let Some(ref json) = ice_servers_json
            && serde_json::from_str::<serde_json::Value>(json).is_err()
        {
            tracing::warn!("ICE_SERVERS is not valid JSON, will be ignored");
        }

        let upload_quota_mb = std::env::var("UPLOAD_QUOTA_MB")
            .unwrap_or_else(|_| "500".to_string())
            .parse()
            .unwrap_or(500u64)
            .clamp(0, 100_000);

        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            jwt_private_key_path: std::env::var("JWT_PRIVATE_KEY_PATH")
                .unwrap_or_else(|_| "./secrets/jwt_private.pem".to_string()),
            jwt_public_key_path: std::env::var("JWT_PUBLIC_KEY_PATH")
                .unwrap_or_else(|_| "./secrets/jwt_public.pem".to_string()),
            totp_encryption_key: std::env::var("TOTP_ENCRYPTION_KEY")
                .context("TOTP_ENCRYPTION_KEY must be set (hex-encoded 256-bit key)")?,
            file_storage_path: std::env::var("FILE_STORAGE_PATH")
                .unwrap_or_else(|_| "./data/files".to_string()),
            max_file_size_mb,
            github_api_token: std::env::var("GITHUB_API_TOKEN").ok(),
            github_repo_owner: std::env::var("GITHUB_REPO_OWNER").ok(),
            github_repo_name: std::env::var("GITHUB_REPO_NAME").ok(),
            admin_username: std::env::var("ADMIN_USERNAME").ok(),
            registration_mode,
            community_creation_mode,
            public_url: std::env::var("PUBLIC_URL").ok(),
            ice_servers_json,
            vapid_private_key: std::env::var("VAPID_PRIVATE_KEY").ok(),
            vapid_public_key: std::env::var("VAPID_PUBLIC_KEY").ok(),
            upload_quota_mb,
        })
    }
}

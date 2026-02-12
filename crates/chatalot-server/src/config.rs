use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub jwt_private_key_path: String,
    pub jwt_public_key_path: String,
    pub totp_encryption_key: Option<String>,
    pub file_storage_path: String,
    pub max_file_size_mb: u64,
    pub forgejo_api_url: Option<String>,
    pub forgejo_api_token: Option<String>,
    pub forgejo_repo_owner: Option<String>,
    pub forgejo_repo_name: Option<String>,
    pub admin_username: Option<String>,
    pub registration_mode: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            jwt_private_key_path: std::env::var("JWT_PRIVATE_KEY_PATH")
                .unwrap_or_else(|_| "./secrets/jwt_private.pem".to_string()),
            jwt_public_key_path: std::env::var("JWT_PUBLIC_KEY_PATH")
                .unwrap_or_else(|_| "./secrets/jwt_public.pem".to_string()),
            totp_encryption_key: std::env::var("TOTP_ENCRYPTION_KEY").ok(),
            file_storage_path: std::env::var("FILE_STORAGE_PATH")
                .unwrap_or_else(|_| "./data/files".to_string()),
            max_file_size_mb: std::env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            forgejo_api_url: std::env::var("FORGEJO_API_URL").ok(),
            forgejo_api_token: std::env::var("FORGEJO_API_TOKEN").ok(),
            forgejo_repo_owner: std::env::var("FORGEJO_REPO_OWNER").ok(),
            forgejo_repo_name: std::env::var("FORGEJO_REPO_NAME").ok(),
            admin_username: std::env::var("ADMIN_USERNAME").ok(),
            registration_mode: std::env::var("REGISTRATION_MODE")
                .unwrap_or_else(|_| "invite_only".to_string()),
        })
    }
}

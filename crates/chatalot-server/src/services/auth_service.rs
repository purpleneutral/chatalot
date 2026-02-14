use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm as JwtAlg, Header};
use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zeroize::Zeroize;

use chatalot_common::api_types::{
    AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, TokenResponse, UserPublic,
};
use chatalot_common::constants::{ACCESS_TOKEN_LIFETIME_SECS, REFRESH_TOKEN_LIFETIME_SECS};
use chatalot_db::repos::{key_repo, registration_invite_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

/// Hash a password with Argon2id.
pub(crate) fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(65536, 3, 4, Some(32))
        .map_err(|e| AppError::Internal(format!("argon2 params: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("password hash failed: {e}")))
}

/// Verify a password against an Argon2id hash.
pub(crate) fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Generate a cryptographically random refresh token (32 bytes) and return (raw, sha256_hash).
fn generate_refresh_token() -> (Vec<u8>, Vec<u8>) {
    let mut token = vec![0u8; 32];
    OsRng.fill_bytes(&mut token);

    let mut hasher = Sha256::new();
    hasher.update(&token);
    let hash = hasher.finalize().to_vec();

    (token, hash)
}

/// Issue a JWT access token.
fn issue_access_token(
    state: &AppState,
    user_id: Uuid,
    username: &str,
    is_admin: bool,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = AccessClaims {
        sub: user_id,
        username: username.to_string(),
        is_admin,
        iat: now,
        exp: now + ACCESS_TOKEN_LIFETIME_SECS,
        jti: Uuid::new_v4(),
    };

    let header = Header::new(JwtAlg::EdDSA);
    jsonwebtoken::encode(&header, &claims, &state.jwt_encoding_key)
        .map_err(|e| AppError::Internal(format!("jwt encode failed: {e}")))
}

fn user_to_public(user: &chatalot_db::models::user::User, is_admin: bool) -> UserPublic {
    UserPublic {
        id: user.id,
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        avatar_url: user.avatar_url.clone(),
        status: user.status.clone(),
        custom_status: user.custom_status.clone(),
        is_admin,
        created_at: Some(user.created_at.to_rfc3339()),
    }
}

/// Validate username format: alphanumeric, underscores, hyphens, dots, 3-32 chars.
fn validate_username(username: &str) -> Result<(), AppError> {
    if username.len() < 3 || username.len() > 32 {
        return Err(AppError::Validation(
            "username must be between 3 and 32 characters".to_string(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(AppError::Validation(
            "username may only contain letters, numbers, underscores, hyphens, and dots".to_string(),
        ));
    }
    if !username.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        return Err(AppError::Validation(
            "username must start with a letter or number".to_string(),
        ));
    }
    if username.ends_with('.') {
        return Err(AppError::Validation(
            "username must not end with a dot".to_string(),
        ));
    }
    if username.contains("..") {
        return Err(AppError::Validation(
            "username must not contain consecutive dots".to_string(),
        ));
    }
    Ok(())
}

/// Validate password complexity: 8-128 chars, at least 1 uppercase, 1 lowercase, 1 digit, 1 special char.
pub(crate) fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }
    if password.len() > 128 {
        return Err(AppError::Validation(
            "password must be at most 128 characters".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(AppError::Validation(
            "password must contain at least one uppercase letter".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(AppError::Validation(
            "password must contain at least one lowercase letter".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation(
            "password must contain at least one digit".to_string(),
        ));
    }
    if !password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err(AppError::Validation(
            "password must contain at least one special character".to_string(),
        ));
    }
    Ok(())
}

/// Basic email format validation.
fn validate_email(email: &str) -> Result<(), AppError> {
    if email.len() > 254 {
        return Err(AppError::Validation("email too long".to_string()));
    }
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() || !parts[1].contains('.') {
        return Err(AppError::Validation("invalid email format".to_string()));
    }
    Ok(())
}

/// Register a new user.
pub async fn register(
    state: &AppState,
    req: RegisterRequest,
    device_name: Option<&str>,
    ip_address: Option<&str>,
) -> Result<AuthResponse, AppError> {
    // Check registration mode
    match state.config.registration_mode.as_str() {
        "closed" => {
            return Err(AppError::Validation(
                "registration is currently disabled".to_string(),
            ));
        }
        "invite_only" => {
            let code = req.invite_code.as_deref().ok_or_else(|| {
                AppError::Validation("an invite code is required to register".to_string())
            })?;
            if code.is_empty() {
                return Err(AppError::Validation(
                    "an invite code is required to register".to_string(),
                ));
            }
            let consumed = registration_invite_repo::validate_and_consume(&state.db, code).await?;
            if consumed.is_none() {
                return Err(AppError::Validation(
                    "invalid, expired, or fully used invite code".to_string(),
                ));
            }
        }
        _ => {} // "open" or anything else — allow registration
    }

    // Validate input
    validate_username(&req.username)?;
    validate_email(&req.email)?;

    validate_password(&req.password)?;
    if req.identity_key.len() != 32 {
        return Err(AppError::Validation(
            "identity key must be 32 bytes".to_string(),
        ));
    }
    if req.signed_prekey.public_key.len() != 32 {
        return Err(AppError::Validation(
            "signed prekey must be 32 bytes".to_string(),
        ));
    }
    if req.signed_prekey.signature.len() != 64 {
        return Err(AppError::Validation(
            "signed prekey signature must be 64 bytes".to_string(),
        ));
    }

    // Check uniqueness
    if user_repo::username_exists(&state.db, &req.username).await? {
        return Err(AppError::Conflict("username already taken".to_string()));
    }
    if user_repo::email_exists(&state.db, &req.email).await? {
        return Err(AppError::Conflict("email already registered".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Compute identity key fingerprint
    let fingerprint = hex::encode(Sha256::digest(&req.identity_key));

    // Create user
    let user_id = Uuid::now_v7();
    let user = user_repo::create_user(
        &state.db,
        user_id,
        &req.username,
        &req.display_name,
        &req.email,
        &password_hash,
        &req.identity_key,
        &fingerprint,
    )
    .await?;

    // Store signed prekey
    key_repo::upsert_signed_prekey(
        &state.db,
        Uuid::now_v7(),
        user_id,
        req.signed_prekey.key_id,
        &req.signed_prekey.public_key,
        &req.signed_prekey.signature,
    )
    .await?;

    // Store one-time prekeys
    if !req.one_time_prekeys.is_empty() {
        let pairs: Vec<(i32, Vec<u8>)> = req
            .one_time_prekeys
            .into_iter()
            .map(|p| (p.key_id, p.public_key))
            .collect();
        key_repo::upload_one_time_prekeys(&state.db, user_id, &pairs).await?;
    }

    // First registered user becomes admin automatically
    let is_admin = if user_repo::count_users(&state.db).await.unwrap_or(1) == 1 {
        user_repo::set_admin(&state.db, user.id, true).await.ok();
        true
    } else {
        user.is_admin
    };

    // Issue tokens
    let access_token = issue_access_token(state, user.id, &user.username, is_admin)?;
    let (mut refresh_raw, refresh_hash) = generate_refresh_token();

    let refresh_id = Uuid::new_v4();
    let expires_at = Utc::now()
        + chrono::Duration::try_seconds(REFRESH_TOKEN_LIFETIME_SECS).unwrap();

    user_repo::create_refresh_token(
        &state.db,
        refresh_id,
        user.id,
        &refresh_hash,
        device_name,
        ip_address,
        expires_at,
    )
    .await?;

    // Audit log
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(user.id),
        "register",
        None,
        None,
        None,
    )
    .await?;

    let refresh_token_hex = hex::encode(&refresh_raw);
    refresh_raw.zeroize();

    Ok(AuthResponse {
        access_token,
        refresh_token: refresh_token_hex,
        user: user_to_public(&user, is_admin),
    })
}

/// Log in an existing user.
pub async fn login(
    state: &AppState,
    req: LoginRequest,
    device_name: Option<&str>,
    ip_address: Option<&str>,
) -> Result<AuthResponse, AppError> {
    let user = user_repo::find_by_username(&state.db, &req.username)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verify password (constant-time via Argon2)
    if !verify_password(&req.password, &user.password_hash)? {
        // Audit failed login
        user_repo::insert_audit_log(
            &state.db,
            Uuid::now_v7(),
            Some(user.id),
            "login_failed",
            None,
            None,
            None,
        )
        .await?;
        return Err(AppError::Unauthorized);
    }

    // TOTP verification
    if user.totp_enabled {
        let code = req.totp_code.as_deref().ok_or_else(|| {
            AppError::Validation("2FA code required".to_string())
        })?;
        let totp_secret = user.totp_secret.as_ref().ok_or_else(|| {
            AppError::Internal("totp_enabled but no secret".to_string())
        })?;
        if !crate::routes::totp::verify_totp_code(
            totp_secret,
            code,
            &state.config.totp_encryption_key,
        )? {
            user_repo::insert_audit_log(
                &state.db,
                Uuid::now_v7(),
                Some(user.id),
                "login_failed_2fa",
                None,
                None,
                None,
            )
            .await?;
            return Err(AppError::Unauthorized);
        }
    }

    // Check suspension
    if user.suspended_at.is_some() {
        return Err(AppError::Validation("account is suspended".to_string()));
    }

    // Issue tokens
    let access_token = issue_access_token(state, user.id, &user.username, user.is_admin)?;
    let (mut refresh_raw, refresh_hash) = generate_refresh_token();

    let refresh_id = Uuid::new_v4();
    let expires_at = Utc::now()
        + chrono::Duration::try_seconds(REFRESH_TOKEN_LIFETIME_SECS).unwrap();

    user_repo::create_refresh_token(
        &state.db,
        refresh_id,
        user.id,
        &refresh_hash,
        device_name,
        ip_address,
        expires_at,
    )
    .await?;

    // Audit successful login
    user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(user.id),
        "login",
        None,
        None,
        None,
    )
    .await?;

    let refresh_token_hex = hex::encode(&refresh_raw);
    refresh_raw.zeroize();

    Ok(AuthResponse {
        access_token,
        refresh_token: refresh_token_hex,
        user: user_to_public(&user, user.is_admin),
    })
}

/// Refresh an access token using a valid refresh token.
pub async fn refresh_token(
    state: &AppState,
    req: RefreshRequest,
    device_name: Option<&str>,
    ip_address: Option<&str>,
) -> Result<TokenResponse, AppError> {
    // Decode the hex refresh token and hash it
    let raw_token =
        hex::decode(&req.refresh_token).map_err(|_| AppError::Unauthorized)?;
    let token_hash = Sha256::digest(&raw_token).to_vec();

    // Look up the refresh token
    let stored = user_repo::find_refresh_token_by_hash(&state.db, &token_hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Revoke the old refresh token (rotation)
    user_repo::revoke_refresh_token(&state.db, stored.id).await?;

    // Look up the user
    let user = user_repo::find_by_id(&state.db, stored.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Check suspension
    if user.suspended_at.is_some() {
        return Err(AppError::Unauthorized);
    }

    // Issue new tokens
    let access_token = issue_access_token(state, user.id, &user.username, user.is_admin)?;
    let (mut refresh_raw, refresh_hash) = generate_refresh_token();

    let refresh_id = Uuid::new_v4();
    let expires_at = Utc::now()
        + chrono::Duration::try_seconds(REFRESH_TOKEN_LIFETIME_SECS).unwrap();

    user_repo::create_refresh_token(
        &state.db,
        refresh_id,
        user.id,
        &refresh_hash,
        device_name,
        ip_address,
        expires_at,
    )
    .await?;

    let refresh_token_hex = hex::encode(&refresh_raw);
    refresh_raw.zeroize();

    Ok(TokenResponse {
        access_token,
        refresh_token: refresh_token_hex,
    })
}

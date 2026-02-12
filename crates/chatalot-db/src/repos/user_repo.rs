use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{IdentityKey, RefreshToken, User};

/// Create a new user with their identity key in a single transaction.
pub async fn create_user(
    pool: &PgPool,
    id: Uuid,
    username: &str,
    display_name: &str,
    email: &str,
    password_hash: &str,
    identity_key: &[u8],
    fingerprint: &str,
) -> Result<User, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, display_name, email, password_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(username)
    .bind(display_name)
    .bind(email)
    .bind(password_hash)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO identity_keys (user_id, identity_key, fingerprint)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(id)
    .bind(identity_key)
    .bind(fingerprint)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(user)
}

/// Find a user by username.
pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
}

/// Find a user by ID.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Check if a username is already taken.
pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(username)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

/// Check if an email is already taken.
pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

/// Get a user's identity key.
pub async fn get_identity_key(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<IdentityKey>, sqlx::Error> {
    sqlx::query_as::<_, IdentityKey>("SELECT * FROM identity_keys WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

/// Store a refresh token hash.
pub async fn create_refresh_token(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    token_hash: &[u8],
    device_name: Option<&str>,
    ip_address: Option<&str>,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, device_name, ip_address, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(device_name)
    .bind(ip_address)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Find a valid (non-revoked, non-expired) refresh token by its hash.
pub async fn find_refresh_token_by_hash(
    pool: &PgPool,
    token_hash: &[u8],
) -> Result<Option<RefreshToken>, sqlx::Error> {
    sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT * FROM refresh_tokens
        WHERE token_hash = $1
          AND revoked_at IS NULL
          AND expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

/// Revoke a refresh token.
pub async fn revoke_refresh_token(pool: &PgPool, token_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Search users by username prefix.
pub async fn search_users(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username ILIKE $1 ORDER BY username ASC LIMIT $2",
    )
    .bind(format!("{query}%"))
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Store a TOTP secret (setup phase, not yet enabled).
pub async fn set_totp_secret(
    pool: &PgPool,
    user_id: Uuid,
    secret: &[u8],
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET totp_secret = $1, updated_at = NOW() WHERE id = $2")
        .bind(secret)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Enable TOTP for a user (after successful verification).
pub async fn enable_totp(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET totp_enabled = true, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Disable TOTP and clear the secret.
pub async fn disable_totp(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET totp_enabled = false, totp_secret = NULL, updated_at = NOW() WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Account Management ──

/// Update a user's password hash.
pub async fn update_password(pool: &PgPool, user_id: Uuid, new_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update profile fields (only non-None values are applied).
pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    display_name: Option<&str>,
    avatar_url: Option<Option<&str>>,
    custom_status: Option<Option<&str>>,
) -> Result<Option<User>, sqlx::Error> {
    // Build dynamic update to only touch provided fields
    let mut set_clauses = vec!["updated_at = NOW()".to_string()];
    let mut param_idx = 2u32; // $1 is user_id

    if display_name.is_some() {
        set_clauses.push(format!("display_name = ${param_idx}"));
        param_idx += 1;
    }
    if avatar_url.is_some() {
        set_clauses.push(format!("avatar_url = ${param_idx}"));
        param_idx += 1;
    }
    if custom_status.is_some() {
        set_clauses.push(format!("custom_status = ${param_idx}"));
    }

    let query = format!(
        "UPDATE users SET {} WHERE id = $1 RETURNING *",
        set_clauses.join(", ")
    );

    let mut q = sqlx::query_as::<_, User>(&query).bind(user_id);
    if let Some(dn) = display_name {
        q = q.bind(dn);
    }
    if let Some(av) = avatar_url {
        q = q.bind(av);
    }
    if let Some(cs) = custom_status {
        q = q.bind(cs);
    }

    q.fetch_optional(pool).await
}

/// Revoke ALL active refresh tokens for a user (server-side logout).
pub async fn revoke_all_refresh_tokens(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// List all active (non-revoked, non-expired) sessions for a user.
pub async fn list_active_sessions(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<RefreshToken>, sqlx::Error> {
    sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT * FROM refresh_tokens
        WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > NOW()
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Delete a user (FK cascades handle tokens, memberships, keys).
pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ── Admin ──

/// Count total users.
pub async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Set a user's admin flag.
pub async fn set_admin(pool: &PgPool, user_id: Uuid, is_admin: bool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET is_admin = $1, updated_at = NOW() WHERE id = $2")
        .bind(is_admin)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Ensure a user is admin by username (for env-var seeding).
pub async fn ensure_admin(pool: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE users SET is_admin = true WHERE username = $1 AND is_admin = false",
    )
    .bind(username)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Suspend a user.
pub async fn suspend_user(
    pool: &PgPool,
    user_id: Uuid,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET suspended_at = NOW(), suspended_reason = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(reason)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Unsuspend a user.
pub async fn unsuspend_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET suspended_at = NULL, suspended_reason = NULL, updated_at = NOW() WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// List all users (paginated, with optional search).
pub async fn list_all_users(
    pool: &PgPool,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<User>, sqlx::Error> {
    if let Some(q) = search {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE username ILIKE $1 OR display_name ILIKE $1 OR email ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(format!("%{q}%"))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

/// Insert an audit log entry.
pub async fn insert_audit_log(
    pool: &PgPool,
    id: Uuid,
    user_id: Option<Uuid>,
    action: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    metadata: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, action, ip_address, user_agent, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(action)
    .bind(ip_address)
    .bind(user_agent)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

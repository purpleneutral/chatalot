use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::registration_invite::RegistrationInvite;

pub async fn create_invite(
    pool: &PgPool,
    id: Uuid,
    code: &str,
    created_by: Uuid,
    max_uses: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<RegistrationInvite, sqlx::Error> {
    sqlx::query_as::<_, RegistrationInvite>(
        r#"
        INSERT INTO registration_invites (id, code, created_by, max_uses, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(code)
    .bind(created_by)
    .bind(max_uses)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// Atomically validate and consume an invite code.
/// Returns the invite if valid, None if invalid/expired/exhausted.
pub async fn validate_and_consume(
    pool: &PgPool,
    code: &str,
) -> Result<Option<RegistrationInvite>, sqlx::Error> {
    sqlx::query_as::<_, RegistrationInvite>(
        r#"
        UPDATE registration_invites
        SET used_count = used_count + 1
        WHERE code = $1
          AND (max_uses IS NULL OR used_count < max_uses)
          AND (expires_at IS NULL OR expires_at > NOW())
        RETURNING *
        "#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn list_invites(pool: &PgPool) -> Result<Vec<RegistrationInvite>, sqlx::Error> {
    sqlx::query_as::<_, RegistrationInvite>(
        "SELECT * FROM registration_invites ORDER BY created_at DESC LIMIT 500",
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_invite(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM registration_invites WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

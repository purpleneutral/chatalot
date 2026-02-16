use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::group::GroupInvite;

pub async fn create_invite(
    pool: &PgPool,
    id: Uuid,
    group_id: Uuid,
    created_by: Uuid,
    code: &str,
    max_uses: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<GroupInvite, sqlx::Error> {
    sqlx::query_as::<_, GroupInvite>(
        r#"
        INSERT INTO group_invites (id, group_id, created_by, code, max_uses, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(group_id)
    .bind(created_by)
    .bind(code)
    .bind(max_uses)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn get_invite_by_code(
    pool: &PgPool,
    code: &str,
) -> Result<Option<GroupInvite>, sqlx::Error> {
    sqlx::query_as::<_, GroupInvite>(
        "SELECT * FROM group_invites WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

/// Atomically increment usage count, returning true if the invite was valid and used.
pub async fn increment_usage(pool: &PgPool, invite_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE group_invites
        SET used_count = used_count + 1
        WHERE id = $1
          AND (max_uses IS NULL OR used_count < max_uses)
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
    )
    .bind(invite_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_group_invites(
    pool: &PgPool,
    group_id: Uuid,
) -> Result<Vec<GroupInvite>, sqlx::Error> {
    sqlx::query_as::<_, GroupInvite>(
        "SELECT * FROM group_invites WHERE group_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
}

pub async fn delete_invite(pool: &PgPool, invite_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM group_invites WHERE id = $1")
        .bind(invite_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

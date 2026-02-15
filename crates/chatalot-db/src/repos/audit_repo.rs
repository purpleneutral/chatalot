use sqlx::PgPool;
use uuid::Uuid;

use crate::models::audit_log::AuditLogEntry;

/// Query audit log with optional filters and pagination.
pub async fn query_audit_log(
    pool: &PgPool,
    action: Option<&str>,
    user_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
    let limit = limit.min(100);

    match (action, user_id) {
        (Some(a), Some(u)) => {
            sqlx::query_as::<_, AuditLogEntry>(
                r#"
                SELECT * FROM audit_log
                WHERE action = $1 AND user_id = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(a)
            .bind(u)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
        (Some(a), None) => {
            sqlx::query_as::<_, AuditLogEntry>(
                r#"
                SELECT * FROM audit_log
                WHERE action = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(a)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
        (None, Some(u)) => {
            sqlx::query_as::<_, AuditLogEntry>(
                r#"
                SELECT * FROM audit_log
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(u)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
        (None, None) => {
            sqlx::query_as::<_, AuditLogEntry>(
                "SELECT * FROM audit_log ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
    }
}

/// Count audit log entries with optional filters.
pub async fn count_audit_log(
    pool: &PgPool,
    action: Option<&str>,
    user_id: Option<Uuid>,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = match (action, user_id) {
        (Some(a), Some(u)) => {
            sqlx::query_as("SELECT COUNT(*) FROM audit_log WHERE action = $1 AND user_id = $2")
                .bind(a)
                .bind(u)
                .fetch_one(pool)
                .await?
        }
        (Some(a), None) => {
            sqlx::query_as("SELECT COUNT(*) FROM audit_log WHERE action = $1")
                .bind(a)
                .fetch_one(pool)
                .await?
        }
        (None, Some(u)) => {
            sqlx::query_as("SELECT COUNT(*) FROM audit_log WHERE user_id = $1")
                .bind(u)
                .fetch_one(pool)
                .await?
        }
        (None, None) => {
            sqlx::query_as("SELECT COUNT(*) FROM audit_log")
                .fetch_one(pool)
                .await?
        }
    };
    Ok(row.0)
}

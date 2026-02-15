use sqlx::PgPool;
use uuid::Uuid;

use crate::models::report::Report;

/// Create a new report.
pub async fn create_report(
    pool: &PgPool,
    id: Uuid,
    reporter_id: Uuid,
    report_type: &str,
    target_id: Uuid,
    reason: &str,
) -> Result<Report, sqlx::Error> {
    sqlx::query_as::<_, Report>(
        r#"
        INSERT INTO reports (id, reporter_id, report_type, target_id, reason)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(reporter_id)
    .bind(report_type)
    .bind(target_id)
    .bind(reason)
    .fetch_one(pool)
    .await
}

/// List reports with optional status filter and pagination.
pub async fn list_reports(
    pool: &PgPool,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Report>, sqlx::Error> {
    if let Some(status) = status {
        sqlx::query_as::<_, Report>(
            "SELECT * FROM reports WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Report>(
            "SELECT * FROM reports ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

/// Count reports with optional status filter.
pub async fn count_reports(pool: &PgPool, status: Option<&str>) -> Result<i64, sqlx::Error> {
    let row: (i64,) = if let Some(status) = status {
        sqlx::query_as("SELECT COUNT(*) FROM reports WHERE status = $1")
            .bind(status)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_as("SELECT COUNT(*) FROM reports")
            .fetch_one(pool)
            .await?
    };
    Ok(row.0)
}

/// Update report status (admin review).
pub async fn review_report(
    pool: &PgPool,
    report_id: Uuid,
    reviewed_by: Uuid,
    status: &str,
    admin_notes: Option<&str>,
) -> Result<Option<Report>, sqlx::Error> {
    sqlx::query_as::<_, Report>(
        r#"
        UPDATE reports
        SET status = $2, reviewed_by = $3, reviewed_at = NOW(), admin_notes = $4
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(report_id)
    .bind(status)
    .bind(reviewed_by)
    .bind(admin_notes)
    .fetch_optional(pool)
    .await
}

/// Get a single report by ID.
pub async fn get_report(pool: &PgPool, id: Uuid) -> Result<Option<Report>, sqlx::Error> {
    sqlx::query_as::<_, Report>("SELECT * FROM reports WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

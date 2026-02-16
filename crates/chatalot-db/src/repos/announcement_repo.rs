use sqlx::PgPool;
use uuid::Uuid;

use crate::models::announcement::Announcement;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    title: &str,
    body: &str,
    created_by: Uuid,
) -> Result<Announcement, sqlx::Error> {
    sqlx::query_as::<_, Announcement>(
        r#"
        INSERT INTO announcements (id, title, body, created_by)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(body)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

pub async fn list_undismissed(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Announcement>, sqlx::Error> {
    sqlx::query_as::<_, Announcement>(
        r#"
        SELECT a.* FROM announcements a
        WHERE a.id NOT IN (
            SELECT announcement_id FROM announcement_dismissals WHERE user_id = $1
        )
        ORDER BY a.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn dismiss(
    pool: &PgPool,
    user_id: Uuid,
    announcement_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO announcement_dismissals (user_id, announcement_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, announcement_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(announcement_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<Announcement>, sqlx::Error> {
    sqlx::query_as::<_, Announcement>("SELECT * FROM announcements ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

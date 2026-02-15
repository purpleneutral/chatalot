use sqlx::PgPool;
use uuid::Uuid;

use crate::models::webhook::Webhook;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    channel_id: Uuid,
    name: &str,
    token: &str,
    created_by: Uuid,
    avatar_url: Option<&str>,
) -> Result<Webhook, sqlx::Error> {
    sqlx::query_as::<_, Webhook>(
        r#"
        INSERT INTO webhooks (id, channel_id, name, token, created_by, avatar_url)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(channel_id)
    .bind(name)
    .bind(token)
    .bind(created_by)
    .bind(avatar_url)
    .fetch_one(pool)
    .await
}

pub async fn get_by_token(pool: &PgPool, token: &str) -> Result<Option<Webhook>, sqlx::Error> {
    sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE token = $1 AND active = true")
        .bind(token)
        .fetch_optional(pool)
        .await
}

pub async fn list_for_channel(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<Vec<Webhook>, sqlx::Error> {
    sqlx::query_as::<_, Webhook>(
        "SELECT * FROM webhooks WHERE channel_id = $1 ORDER BY created_at ASC",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    avatar_url: Option<Option<&str>>,
    active: Option<bool>,
) -> Result<Option<Webhook>, sqlx::Error> {
    sqlx::query_as::<_, Webhook>(
        r#"
        UPDATE webhooks
        SET name = COALESCE($2, name),
            avatar_url = COALESCE($3, avatar_url),
            active = COALESCE($4, active)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(avatar_url)
    .bind(active)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM webhooks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Webhook>, sqlx::Error> {
    sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

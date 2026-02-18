use sqlx::PgPool;
use uuid::Uuid;

use crate::models::push_subscription::PushSubscription;

/// Insert or update a push subscription (upsert on user_id + endpoint).
pub async fn upsert_subscription(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    endpoint: &str,
    p256dh_key: &str,
    auth_key: &str,
    user_agent: Option<&str>,
) -> Result<PushSubscription, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh_key, auth_key, user_agent)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id, endpoint)
        DO UPDATE SET p256dh_key = $4, auth_key = $5, user_agent = $6, failure_count = 0
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(endpoint)
    .bind(p256dh_key)
    .bind(auth_key)
    .bind(user_agent)
    .fetch_one(pool)
    .await
}

/// Get all subscriptions for a user.
pub async fn get_subscriptions_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<PushSubscription>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM push_subscriptions WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

/// Delete a subscription by endpoint.
pub async fn delete_by_endpoint(
    pool: &PgPool,
    user_id: Uuid,
    endpoint: &str,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM push_subscriptions WHERE user_id = $1 AND endpoint = $2")
            .bind(user_id)
            .bind(endpoint)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete all subscriptions for a user.
pub async fn delete_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM push_subscriptions WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Increment failure count and return the new value.
pub async fn increment_failure_count(
    pool: &PgPool,
    subscription_id: Uuid,
) -> Result<i32, sqlx::Error> {
    let (count,): (i32,) = sqlx::query_as(
        "UPDATE push_subscriptions SET failure_count = failure_count + 1 WHERE id = $1 RETURNING failure_count",
    )
    .bind(subscription_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Mark a subscription as successfully used (resets failure count).
pub async fn mark_used(pool: &PgPool, subscription_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE push_subscriptions SET last_used = NOW(), failure_count = 0 WHERE id = $1",
    )
    .bind(subscription_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete subscriptions with 3+ consecutive failures.
pub async fn cleanup_failed(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM push_subscriptions WHERE failure_count >= 3")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Delete subscriptions not used in 90 days.
pub async fn cleanup_stale(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM push_subscriptions WHERE last_used IS NOT NULL AND last_used < NOW() - INTERVAL '90 days'",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

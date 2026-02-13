use sqlx::PgPool;
use uuid::Uuid;

/// Get preferences for a user. Returns empty JSON object if no row exists.
pub async fn get_preferences(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<serde_json::Value, sqlx::Error> {
    let row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT preferences FROM user_preferences WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0).unwrap_or(serde_json::json!({})))
}

/// Merge partial preferences into existing ones using PostgreSQL's `||` operator.
/// Uses UPSERT so the first call creates the row.
pub async fn merge_preferences(
    pool: &PgPool,
    user_id: Uuid,
    partial: &serde_json::Value,
) -> Result<serde_json::Value, sqlx::Error> {
    let row: (serde_json::Value,) = sqlx::query_as(
        r#"
        INSERT INTO user_preferences (user_id, preferences, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (user_id)
        DO UPDATE SET
            preferences = user_preferences.preferences || $2,
            updated_at = NOW()
        RETURNING preferences
        "#,
    )
    .bind(user_id)
    .bind(partial)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

use sqlx::PgPool;

#[derive(sqlx::FromRow)]
pub struct SettingRow {
    pub key: String,
    pub value: String,
}

pub async fn get(pool: &PgPool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM instance_settings WHERE key = $1")
            .bind(key)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

pub async fn set(pool: &PgPool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO instance_settings (key, value, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<SettingRow>, sqlx::Error> {
    sqlx::query_as::<_, SettingRow>(
        "SELECT key, value FROM instance_settings ORDER BY key",
    )
    .fetch_all(pool)
    .await
}

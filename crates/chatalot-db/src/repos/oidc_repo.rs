use sqlx::PgPool;
use uuid::Uuid;

/// Find a user by OIDC provider + subject. Returns (user_id, username) if found.
pub async fn find_by_provider_subject(
    pool: &PgPool,
    provider: &str,
    subject: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String)>(
        r#"
        SELECT oi.user_id, u.username
        FROM oidc_identities oi
        JOIN users u ON u.id = oi.user_id
        WHERE oi.provider = $1 AND oi.subject = $2
        "#,
    )
    .bind(provider)
    .bind(subject)
    .fetch_optional(pool)
    .await
}

/// Link an OIDC identity to an existing user.
pub async fn link_identity(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    subject: &str,
    email: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO oidc_identities (user_id, provider, subject, email)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (provider, subject) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(subject)
    .bind(email)
    .execute(pool)
    .await?;
    Ok(())
}

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::key_bundle::{OneTimePrekey, SignedPrekey};
use crate::models::user::IdentityKey;

/// Upsert an identity key (for users who registered before E2E was active).
pub async fn upsert_identity_key(
    pool: &PgPool,
    user_id: Uuid,
    identity_key: &[u8],
    fingerprint: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO identity_keys (user_id, identity_key, fingerprint)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id) DO UPDATE
            SET identity_key = EXCLUDED.identity_key,
                fingerprint = EXCLUDED.fingerprint,
                rotated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(identity_key)
    .bind(fingerprint)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upload a signed prekey.
pub async fn upsert_signed_prekey(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    key_id: i32,
    public_key: &[u8],
    signature: &[u8],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO signed_prekeys (id, user_id, key_id, public_key, signature)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id, key_id) DO UPDATE
            SET public_key = EXCLUDED.public_key,
                signature = EXCLUDED.signature,
                created_at = NOW()
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(key_id)
    .bind(public_key)
    .bind(signature)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upload a batch of one-time prekeys.
pub async fn upload_one_time_prekeys(
    pool: &PgPool,
    user_id: Uuid,
    prekeys: &[(i32, Vec<u8>)], // (key_id, public_key)
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (key_id, public_key) in prekeys {
        sqlx::query(
            r#"
            INSERT INTO one_time_prekeys (id, user_id, key_id, public_key)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, key_id) DO NOTHING
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(key_id)
        .bind(public_key)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

/// Fetch a user's key bundle for X3DH (consumes one one-time prekey).
pub async fn fetch_key_bundle(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<KeyBundle>, sqlx::Error> {
    let identity =
        sqlx::query_as::<_, IdentityKey>("SELECT * FROM identity_keys WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    let identity = match identity {
        Some(ik) => ik,
        None => return Ok(None),
    };

    let signed_prekey = sqlx::query_as::<_, SignedPrekey>(
        "SELECT * FROM signed_prekeys WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let signed_prekey = match signed_prekey {
        Some(spk) => spk,
        None => return Ok(None),
    };

    // Atomically claim one unused one-time prekey
    let one_time_prekey = sqlx::query_as::<_, OneTimePrekey>(
        r#"
        UPDATE one_time_prekeys SET used = TRUE
        WHERE id = (
            SELECT id FROM one_time_prekeys
            WHERE user_id = $1 AND NOT used
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING *
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(Some(KeyBundle {
        identity_key: identity.identity_key,
        signed_prekey,
        one_time_prekey,
    }))
}

/// Fetch just the identity key bytes for a user.
pub async fn fetch_identity_key(pool: &PgPool, user_id: Uuid) -> Result<Option<Vec<u8>>, sqlx::Error> {
    let row: Option<(Vec<u8>,)> =
        sqlx::query_as("SELECT identity_key FROM identity_keys WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

/// Delete ALL one-time prekeys for a user (used during full key re-registration).
pub async fn delete_all_prekeys(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM one_time_prekeys WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Count remaining unused one-time prekeys for a user.
pub async fn count_unused_prekeys(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM one_time_prekeys WHERE user_id = $1 AND NOT used")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

/// A complete key bundle for X3DH.
pub struct KeyBundle {
    pub identity_key: Vec<u8>,
    pub signed_prekey: SignedPrekey,
    pub one_time_prekey: Option<OneTimePrekey>,
}

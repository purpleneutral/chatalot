use sqlx::PgPool;
use uuid::Uuid;

use crate::models::voice::VoiceSession;

/// Get or create an active voice session for a channel.
/// Uses a CTE to atomically check-then-insert, avoiding TOCTOU races
/// where two concurrent callers could create duplicate sessions.
pub async fn get_or_create_session(
    pool: &PgPool,
    channel_id: Uuid,
    started_by: Uuid,
) -> Result<VoiceSession, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query_as::<_, VoiceSession>(
        r#"
        WITH existing AS (
            SELECT * FROM voice_sessions
            WHERE channel_id = $2 AND ended_at IS NULL
            ORDER BY started_at DESC
            LIMIT 1
        ), inserted AS (
            INSERT INTO voice_sessions (id, channel_id, started_by)
            SELECT $1, $2, $3
            WHERE NOT EXISTS (SELECT 1 FROM existing)
            RETURNING *
        )
        SELECT * FROM existing
        UNION ALL
        SELECT * FROM inserted
        LIMIT 1
        "#,
    )
    .bind(id)
    .bind(channel_id)
    .bind(started_by)
    .fetch_one(pool)
    .await
}

/// Get the active voice session for a channel (if any).
pub async fn get_active_session(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<Option<VoiceSession>, sqlx::Error> {
    sqlx::query_as::<_, VoiceSession>(
        "SELECT * FROM voice_sessions WHERE channel_id = $1 AND ended_at IS NULL ORDER BY started_at DESC LIMIT 1",
    )
    .bind(channel_id)
    .fetch_optional(pool)
    .await
}

/// Add a participant to a voice session.
pub async fn join_session(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO voice_session_participants (session_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (session_id, user_id) DO UPDATE SET left_at = NULL, joined_at = NOW()
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a participant from a voice session.
pub async fn leave_session(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE voice_session_participants SET left_at = NOW() WHERE session_id = $1 AND user_id = $2 AND left_at IS NULL",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get current participants in a voice session.
pub async fn get_participants(pool: &PgPool, session_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM voice_session_participants WHERE session_id = $1 AND left_at IS NULL",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Remove a user from all active voice sessions they're in.
/// Returns (session_id, channel_id) pairs for each session they were removed from.
pub async fn leave_all_sessions(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<(Uuid, Uuid)>, sqlx::Error> {
    let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        UPDATE voice_session_participants vsp
        SET left_at = NOW()
        FROM voice_sessions vs
        WHERE vsp.session_id = vs.id
          AND vsp.user_id = $1
          AND vsp.left_at IS NULL
          AND vs.ended_at IS NULL
        RETURNING vs.id, vs.channel_id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Remove a user from voice sessions they joined before a given cutoff time.
/// Used by the grace period cleanup: if the user rejoined after the cutoff,
/// their new session record is preserved.
pub async fn leave_sessions_joined_before(
    pool: &PgPool,
    user_id: Uuid,
    cutoff: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<(Uuid, Uuid)>, sqlx::Error> {
    let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        UPDATE voice_session_participants vsp
        SET left_at = NOW()
        FROM voice_sessions vs
        WHERE vsp.session_id = vs.id
          AND vsp.user_id = $1
          AND vsp.left_at IS NULL
          AND vsp.joined_at <= $2
          AND vs.ended_at IS NULL
        RETURNING vs.id, vs.channel_id
        "#,
    )
    .bind(user_id)
    .bind(cutoff)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Check if two users are in the same active voice session.
pub async fn are_in_same_session(
    pool: &PgPool,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT 1 FROM voice_session_participants a
        JOIN voice_session_participants b ON a.session_id = b.session_id
        JOIN voice_sessions s ON s.id = a.session_id
        WHERE a.user_id = $1 AND b.user_id = $2
          AND a.left_at IS NULL AND b.left_at IS NULL
          AND s.ended_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(user_a)
    .bind(user_b)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

/// End a voice session (when last participant leaves).
pub async fn end_session(pool: &PgPool, session_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE voice_sessions SET ended_at = NOW() WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

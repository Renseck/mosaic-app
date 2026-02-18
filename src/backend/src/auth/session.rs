use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Returned by `validate_session` - raw strings so this module stays independent of middleware types.
pub struct SessionUser {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
}

/* ============================================================================================== */
///Generates a cryptographically random 32-byte hex session token.
pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    hex::encode(bytes)
}

/* ============================================================================================== */
/// SHA-256 hashes a raw token for storage/lookup.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/* ============================================================================================== */
/// Extracts the `portal_session` cookie value from request headers (returns None if absent).
pub fn extract_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|seg| {
                seg.trim()
                    .strip_prefix("portal_session=")
                    .map(|v| v.to_owned())
            })
        })
}

/* ============================================================================================== */
/// Inserts a new session row and returns the raw (unhashed) token to set as a cookie.
pub async fn create_session(pool: &PgPool, user_id: Uuid, ttl_hours: u64) -> Result<String, AppError> {
    let token = generate_session_token();
    let token_hash = hash_token(&token);
    let expires_at = Utc::now() + Duration::hours(ttl_hours as i64);

    sqlx::query!(
        "INSERT INTO portal.sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user_id,
        token_hash,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(token)
}

/* ============================================================================================== */
/// Validates a raw token against the DB, returning the associated user or Unauthorized.
pub async fn validate_session(pool: &PgPool, token: &str) -> Result<SessionUser, AppError> {
    let token_hash = hash_token(token);

    let row = sqlx::query!(
        r#"
        SELECT u.id AS user_id, u.username, u.role
        FROM portal.sessions s
        JOIN portal.users u ON s.user_id = u.id
        WHERE s.token_hash = $1 AND s.expires_at > now()
        "#,
        token_hash
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok(SessionUser {
        user_id: row.user_id,
        username: row.username,
        role: row.role,
    })
}

/* ============================================================================================== */

/// Deletes a session by raw token (called on logout).
pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    let token_hash = hash_token(token);
    sqlx::query!(
        "DELETE FROM portal.sessions WHERE token_hash = $1",
        token_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}
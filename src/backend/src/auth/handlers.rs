use axum::{
    Json, extract::State, http::{HeaderMap, HeaderValue, StatusCode}, response::IntoResponse
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{middleware::AuthenticatedUser, password, session};
use crate::error::AppError;
use crate::AppState;

/* ============================================================================================== */
/*                                              DTOs                                              */
/* ============================================================================================== */

#[derive(Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

/* ============================================================================================== */
/*                                         Cookie helpers                                         */
/* ============================================================================================== */

fn build_session_cookie(token: &str, ttl_hours: u64) -> HeaderValue {
    let max_age = ttl_hours * 3600;
    HeaderValue::from_str(&format!(
        "portal_session={token}; HttpOnly; SameSite=Strict; Max-Age={max_age}; Path=/"
    ))
    .expect("cookie string is always valid ASCII")
}

const CLEAR_COOKIE: &str = "portal_session=; HttpOnly; SameSite=Strict; Max-Age=0; Path=/";

/* ============================================================================================== */
/*                                            Handlers                                            */
/* ============================================================================================== */

/// POST /api/auth/register
///
/// First caller (no existing users) is created as admin — no auth required.
/// Subsequent registrations require a valid admin session cookie.
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RegisterInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.username.trim().is_empty() {
        return Err(AppError::Validation(
            "username is required".into(),
        ));
    }
    if input.password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }

    let user_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM portal.users")
        .fetch_one(&state.pool)
        .await?
        .unwrap_or(0);

    let role = if user_count == 0 {
        "admin"
    } else {
        // Non-first user: Caller must be an authenticated admin.
        let token = session::extract_cookie(&headers).ok_or(AppError::Unauthorized)?;
        let caller = session::validate_session(&state.pool, &token).await?;
        if caller.role != "admin" {
            return Err(AppError::Forbidden);
        }
        "viewer"
    };

    let password_hash = password::hash_password(&input.password)?;

    let user = sqlx::query!(
        r#"
        INSERT INTO portal.users (username, email, password_hash, role)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, role, created_at
        "#,
        input.username.trim(),
        input.email,
        password_hash,
        role,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("users_username_key") {
                return AppError::Validation("username already taken".into());
            }
        }
        AppError::Database(e)
    })?;

    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
        }),
    ))
}

/* ============================================================================================== */
/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch user - Generic error to prevent username enumeration.
    let user = sqlx::query!(
        "
        SELECT id, username, email, password_hash, role, created_at
        FROM portal.users
        WHERE username = $1
        ",
        input.username
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !password::verify_password(&input.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let token = session::create_session(&state.pool, user.id, state.config.session_ttl_hours).await?;

    let mut response = Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at,
    })
    .into_response();

    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        build_session_cookie(&token, state.config.session_ttl_hours),
    );

    Ok(response)
}

/* ============================================================================================== */
/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    if let Some(token) = session::extract_cookie(&headers) {
        session::delete_session(&state.pool, &token).await?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_static(CLEAR_COOKIE),
    );
    Ok(response)
}

/* ============================================================================================== */
/// GET /api/auth/me — returns the current user; 401 if no valid session.
pub async fn me(user: AuthenticatedUser) -> impl IntoResponse {
    Json(serde_json::json!({
        "id": user.user_id,
        "username": user.username,
        "role": user.role,
    }))
}

/* ============================================================================================== */
/// POST /api/auth/change-password — changes the authenticated user's password.
#[derive(Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<ChangePasswordInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.new_password.len() < 8 {
        return Err(AppError::Validation(
            "new password must be at least 8 characters".into(),
        ));
    }

    let row = sqlx::query!(
        "SELECT password_hash FROM portal.users WHERE id = $1",
        user.user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    if !password::verify_password(&input.current_password, &row.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let new_hash = password::hash_password(&input.new_password)?;
    sqlx::query!(
        "UPDATE portal.users SET password_hash = $1, updated_at = now() WHERE id = $2",
        new_hash,
        user.user_id
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
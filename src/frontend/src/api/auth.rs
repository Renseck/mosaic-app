use serde::Serialize;
use crate::models::User;
use super::client::{self, ApiError};

#[derive(Serialize)]
struct LoginBody<'a> {
    username: &'a str,
    password: &'a str,
}

/* ============================================================================================== */
/// POST /api/auth/login - returns the logged-in user on success.
pub async fn login(username: &str, password: &str) -> Result<User, ApiError> {
    client::post_json("/api/auth/login", &LoginBody { username, password }).await
}

/* ============================================================================================== */
/// POST /api/auth/logout - clears the session cookie.
pub async fn logout() -> Result<(), ApiError> {
    client::post_empty("/api/auth/logout").await
}

/* ============================================================================================== */
/// GET /api/auth/me - returns current user or ApiErro::Server(401).
pub async fn me() -> Result<User, ApiError> {
    client::get("/api/auth/me").await
}

/* ============================================================================================== */
#[derive(Serialize)]
struct ChangePasswordBody<'a> {
    current_password: &'a str,
    new_password: &'a str,
}

/// POST /api/auth/change-password — 204 on success.
pub async fn change_password(current: &str, new: &str) -> Result<(), ApiError> {
    client::post_json_empty(
        "/api/auth/change-password",
        &ChangePasswordBody { current_password: current, new_password: new },
    )
    .await
}
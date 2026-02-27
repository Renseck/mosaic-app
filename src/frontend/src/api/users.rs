use serde::Serialize;
use crate::models::User;
use super::client::{self, ApiError};

/* ============================================================================================== */
/// GET /api/users — admin-only list of all users.
pub async fn list_users() -> Result<Vec<User>, ApiError> {
    client::get("/api/users").await
}

/* ============================================================================================== */
#[derive(Serialize)]
struct UpdateRoleBody {
    role: String,
}

/// PUT /api/users/:id/role — changes a user's role (admin only).
pub async fn update_user_role(id: &str, role: &str) -> Result<User, ApiError> {
    client::put_json(
        &format!("/api/users/{id}/role"),
        &UpdateRoleBody { role: role.to_string() },
    )
    .await
}

/* ============================================================================================== */
#[derive(Serialize)]
struct RegisterBody<'a> {
    username: &'a str,
    password: &'a str,
    email: Option<&'a str>,
}

/// POST /api/auth/register — creates a new user (admin-only after bootstrap).
pub async fn create_user(
    username: &str,
    password: &str,
    email: Option<&str>,
) -> Result<User, ApiError> {
    client::post_json(
        "/api/auth/register",
        &RegisterBody { username, password, email },
    )
    .await
}

/* ============================================================================================== */
#[derive(Serialize)]
struct ResetPasswordBody<'a> {
    new_password: &'a str,
}

/// PUT /api/users/:id/password — admin resets a user's password.
pub async fn reset_user_password(id: &str, new_password: &str) -> Result<(), ApiError> {
    client::put_json_empty(
        &format!("/api/users/{id}/password"),
        &ResetPasswordBody { new_password },
    )
    .await
}
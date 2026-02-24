use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{middleware::{AuthenticatedUser, RequireAdmin, Role}, password};
use crate::error::AppError;
use crate::AppState;

#[derive(Deserialize)]
pub struct UpdateRoleInput {
    pub role: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password:     String,
}

/* ============================================================================================== */
///GET    /api/users — lists all users (admin only).
pub async fn list_users(
    State(state): State<AppState>,
    _admin: RequireAdmin,
) -> Result<impl IntoResponse, AppError> {
    let users = state.users.list().await?;
    Ok(Json(users))
}

/* ============================================================================================== */
///PUT    /api/users/:id/role — changes a user's role (admin only).
pub async fn update_user_role(
    State(state): State<AppState>,
    _admin: RequireAdmin,
    Path(user_id): Path<Uuid>,
    Json(input): Json<UpdateRoleInput>,
) -> Result<impl IntoResponse, AppError> {
    // Validate the role value before touching the DB.
    Role::try_from(input.role.as_str())?;
    let user = state.users.update_role(user_id, &input.role).await?;
    Ok(Json(user))
}
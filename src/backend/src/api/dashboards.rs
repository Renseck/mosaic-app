use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::auth::middleware::{AuthenticatedUser, Role};
use crate::db::repos::dashboard_repo::{CreateDashboard, UpdateDashboard};
use crate::error::AppError;
use crate::AppState;

/* ============================================================================================== */
/// GET /api/dashboards - lists dashboards owned by the caller or marked shared. 
pub async fn list_dashboards(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let dashboards = state.dashboards.list_for_user(user.user_id).await?;
    Ok(Json(dashboards))
}

/* ============================================================================================== */
/// GET    /api/dashboards/:slug
pub async fn get_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let dashboard = state.dashboards.get_by_slug(&slug).await?;
    
    // Viewers can only access shared dashboards or their own.
    if !dashboard.is_shared
        && dashboard.owner_id != Some(user.user_id)
        && user.role != Role::Admin
    {
        return Err(AppError::Forbidden);
    }

    let panels = state.panels.list_for_dashboard(dashboard.id).await?;
    Ok(Json(serde_json::json!({ "dashboard": dashboard, "panels": panels })))
}

/* ============================================================================================== */
/// POST   /api/dashboards
pub async fn create_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<CreateDashboard>,
) -> Result<impl IntoResponse, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("title is required".into()));
    }
    let dashboard = state.dashboards.create(user.user_id, input).await?;
    Ok((StatusCode::CREATED, Json(dashboard)))
}

/* ============================================================================================== */
/// PUT    /api/dashboards/:id
pub async fn update_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(dasboard_id): Path<Uuid>,
    Json(input): Json<UpdateDashboard>,
) -> Result<impl IntoResponse, AppError> {
    let dashboard = state.dashboards.get_by_id(dasboard_id).await?;
    require_owner_or_admin(&dashboard.owner_id, &user)?;
    let updated = state.dashboards.update(dasboard_id, input).await?;
    Ok(Json(updated))
}

/* ============================================================================================== */
/// DELETE /api/dashboards/:id
pub async fn delete_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let dashboard = state.dashboards.get_by_id(id).await?;
    require_owner_or_admin(&dashboard.owner_id, &user)?;
    state.dashboards.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/* ============================================================================================== */
/*                                             Helpers                                            */
/* ============================================================================================== */

pub(crate) fn require_owner_or_admin(
    owner_id: &Option<Uuid>,
    user: &AuthenticatedUser,
) -> Result<(), AppError> {
    if *owner_id != Some(user.user_id) && user.role != Role::Admin {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
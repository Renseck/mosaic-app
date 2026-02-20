use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::repos::panel_repo::{BatchPositionUpdate, CreatePanel, GridPosition, UpdatePanel};
use crate::error::AppError;
use crate::AppState;

use super::dashboards::require_owner_or_admin;

/* ============================================================================================== */
/*                   Handlers mounted under /api/dashboards/:dashboard_id/panels                  */
/* ============================================================================================== */

/// GET    /api/dashboards/:id/panels
pub async fn list_panels(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(dashboard_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let dashboard = state.dashboards.get_by_id(dashboard_id).await?;
    // Read access: shared dashboards are visible to all; private ones require ownership/admin.
    if !dashboard.is_shared
        && dashboard.owner_id != Some(user.user_id)
        && user.role != crate::auth::middleware::Role::Admin
    {
        return Err(AppError::Forbidden);
    }
    let panels = state.panels.list_for_dashboard(dashboard_id).await?;
    Ok(Json(panels))
}

/* ============================================================================================== */
///POST   /api/dashboards/:id/panels
pub async fn create_panel(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(dashboard_id): Path<Uuid>,
    Json(input): Json<CreatePanel>,
) -> Result<impl IntoResponse, AppError> {
    if input.panel_type.trim().is_empty() {
        return Err(AppError::Validation("panel_type is required".into()));
    }
    let dashboard = state.dashboards.get_by_id(dashboard_id).await?;
    require_owner_or_admin(&dashboard.owner_id, &user)?;
    let panel = state.panels.create(dashboard_id, input).await?;
    Ok((StatusCode::CREATED, Json(panel)))
}

/* ============================================================================================== */
/*                               Handlers mounted under /api/panels                               */
/* ============================================================================================== */

/// PUT    /api/panels/batch-position — atomically update grid positions for multiple panels.
pub async fn batch_update_positions(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(updates): Json<Vec<BatchPositionUpdate>>,
) -> Result<impl IntoResponse, AppError> {
    if updates.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }
    state.panels.batch_update_positions(updates).await?;
    Ok(StatusCode::NO_CONTENT)
}

/* ============================================================================================== */
/// PUT    /api/panels/:id — updates panel metadata (title, type, source_url, config).
pub async fn update_panel(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(panel_id): Path<Uuid>,
    Json(input): Json<UpdatePanel>,
) -> Result<impl IntoResponse, AppError> {
    let panel = state.panels.get_by_id(panel_id).await?;
    let dashboard = state.dashboards.get_by_id(panel.dashboard_id).await?;
    require_owner_or_admin(&dashboard.owner_id, &user)?;
    let updated = state.panels.update(panel_id, input).await?;
    Ok(Json(updated))
}

/* ============================================================================================== */
/// PUT    /api/panels/:id/position — updates a single panel's grid position.
pub async fn update_position(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(panel_id): Path<Uuid>,
    Json(pos): Json<GridPosition>,
) -> Result<impl IntoResponse, AppError> {
    let panel = state.panels.get_by_id(panel_id).await?;
    let dashboard = state.dashboards.get_by_id(panel.dashboard_id).await?;
    require_owner_or_admin(&dashboard.owner_id, &user)?;
    let updated = state.panels.update_position(panel_id, pos).await?;
    Ok(Json(updated))
}

/* ============================================================================================== */
/// DELETE /api/panels/:id
pub async fn delete_panel(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(panel_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let panel = state.panels.get_by_id(panel_id).await?;
    let dashboard = state.dashboards.get_by_id(panel.dashboard_id).await?;
    require_owner_or_admin(&dashboard.owner_id, &user)?;
    state.panels.delete(panel_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
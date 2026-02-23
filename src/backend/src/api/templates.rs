use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::repos::template_repo::FieldDefinition;
use crate::error::AppError;
use crate::orchestrator::CreateTemplateInput;
use crate::AppState;

/* ============================================================================================== */
/*                                              DTOs                                              */
/* ============================================================================================== */

#[derive(Deserialize)]
pub struct CreateTemplateRequest {
    pub name:           String,
    pub description:    Option<String>,
    pub fields:         Vec<FieldDefinition>,
}

/* ============================================================================================== */
/*                                            Handlers                                            */
/* ============================================================================================== */

/// GET /api/templates
pub async fn list_templates(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let templates = state.templates.list_all().await?;
    Ok(Json(templates))
}

/* ============================================================================================== */
/// GET /api/templates/:id
pub async fn get_template(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let template = state.templates.get_by_id(id).await?;
    Ok(Json(template))
}

/* ============================================================================================== */
/// POST /api/templates — triggers the full provisioning pipeline
pub async fn create_template(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }

    if req.fields.is_empty() {
        return Err(AppError::Validation("at least one field is required".into()));
    }

    // Validate field names: lowercase alphanumeric + underscore
    for field in &req.fields {
        if !field.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(AppError::Validation(
                format!("field name '{}' must be lowercase alphanumeric + underscore", field.name)
            ));
        }
    }

    let template = state.orchestrator.provision_dataset(
        CreateTemplateInput {
            name:           req.name,
            description:    req.description,
            fields:         req.fields,
        }, 
        user.user_id,
    ).await?;

    Ok((StatusCode::CREATED, Json(template)))
}

/* ============================================================================================== */
/// DELETE /api/templates/:id — removes DB record + cleans up external resources
pub async fn delete_template(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != crate::auth::middleware::Role::Admin {
        return Err(AppError::Forbidden);
    }
    let template = state.templates.get_by_id(id).await?;
    // Best-effort cleanup of external resources
    state.orchestrator.deprovision_dataset(&template).await;
    state.templates.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
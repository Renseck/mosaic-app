use crate::models::template::{CreateTemplateRequest, DatasetTemplate};
use super::client::{self, ApiError};

pub async fn list_templates() -> Result<Vec<DatasetTemplate>, ApiError> {
    client::get("/api/templates").await
}

/* ============================================================================================== */
pub async fn get_template(id: &str) -> Result<DatasetTemplate, ApiError> {
    client::get(&format!("/api/templates/{id}")).await
}

/* ============================================================================================== */
pub async fn create_template(req: &CreateTemplateRequest) -> Result<DatasetTemplate, ApiError> {
    client::post_json("/api/templates", req).await
}

/* ============================================================================================== */
pub async fn delete_template(id: &str) -> Result<(), ApiError> {
    client::delete(&format!("/api/templates/{id}")).await
}
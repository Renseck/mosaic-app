use crate::models::dashboard::{
    BatchPositionUpdate, CreateDashboard, CreatePanel,
    Dashboard, DashboardWithPanels, Panel,
};
use super::client::{self, ApiError};

pub async fn list_dashboards() -> Result<Vec<Dashboard>, ApiError> {
    client::get("/api/dashboards").await
}

/* ============================================================================================== */
pub async fn get_dashboard(slug: &str) -> Result<DashboardWithPanels, ApiError> {
    client::get(&format!("/api/dashboards/{slug}")).await
}

/* ============================================================================================== */
pub async fn create_dashboard(input: &CreateDashboard) -> Result<Dashboard, ApiError> {
    client::post_json("/api/dashboards", input).await
}

/* ============================================================================================== */
pub async fn delete_dashboard(id: &str) -> Result<(), ApiError> {
    client::delete(&format!("/api/dashboards/{id}")).await
}

/* ============================================================================================== */
pub async fn create_panel(dashboard_id: &str, input: &CreatePanel) -> Result<Panel, ApiError> {
    client::post_json(&format!("/api/dashboards/{dashboard_id}/panels"), input).await
}

/* ============================================================================================== */
pub async fn delete_panel(panel_id: &str) -> Result<(), ApiError> {
    client::delete(&format!("/api/panels/{panel_id}")).await
}

/* ============================================================================================== */
pub async fn batch_update_positions(updates: &[BatchPositionUpdate]) -> Result<(), ApiError> {
    client::put_json_empty("/api/panels/batch-position", &updates).await
}
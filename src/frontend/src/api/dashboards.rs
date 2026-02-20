use crate::models::Dashboard;
use super::client::{self, ApiError};

pub async fn list_dashboards() -> Result<Vec<Dashboard>, ApiError> {
    client::get("/api/dashboards").await
}
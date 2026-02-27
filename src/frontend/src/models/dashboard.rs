use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Dashboard {
    pub id:         String,
    pub owner_id:   Option<String>,
    pub title:      String,
    pub slug:       String,
    pub icon:       Option<String>,
    pub sort_order: i32,
    pub is_shared:  bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Panel {
    pub id:           String,
    pub dashboard_id: String,
    pub title:        Option<String>,
    pub panel_type:   String,    // "grafana_panel" | "grafana_dashboard" | "nocodb_form" | "nocodb_grid" | "markdown"
    pub source_url:   Option<String>,
    pub config:       serde_json::Value,
    pub grid_x:       i32,
    pub grid_y:       i32,
    pub grid_w:       i32,
    pub grid_h:       i32,
}

/// Returned by GET /api/dashboards/:slug
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DashboardWithPanels {
    pub dashboard: Dashboard,
    pub panels:    Vec<Panel>,
}

/* ============================================================================================== */
/*                                          Request types                                         */
/* ============================================================================================== */

#[derive(Debug, Serialize)]
pub struct CreateDashboard {
    pub title:      String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon:       Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_shared:  Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateDashboard {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_shared: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CreatePanel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title:      Option<String>,
    pub panel_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub config:     Option<serde_json::Value>,
    pub grid_x:     i32,
    pub grid_y:     i32,
    pub grid_w:     i32,
    pub grid_h:     i32,
}

#[derive(Debug, Serialize)]
pub struct BatchPositionUpdate {
    pub id:     String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_w: i32,
    pub grid_h: i32,
}
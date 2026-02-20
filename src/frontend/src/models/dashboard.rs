use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_shared: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Panel {
    pub id: String,
    pub dashboard_id: String,
    pub title: Option<String>,
    pub panel_type: String,
    pub source_url: Option<String>,
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_w: i32,
    pub grid_h: i32,
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name:       String,
    pub field_type: String,         // "number" | "text" | "date" | "select" 
    pub unit:       Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DatasetTemplate {
    pub id:                    String,
    pub name:                  String,
    pub description:           Option<String>,
    pub nocodb_table_id:       Option<String>,
    pub nocodb_form_id:        Option<String>,
    pub grafana_dashboard_uid: Option<String>,
    pub fields:                serde_json::Value,
    pub created_at:            String,
}

#[derive(Debug, Serialize)]
pub struct CreateTemplateRequest {
    pub name:        String,
    pub description: Option<String>,
    pub fields:      Vec<FieldDefinition>,
}
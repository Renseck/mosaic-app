use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::repos::template_repo::FieldDefinition;
use crate::error::AppError;

pub struct GrafanaClient {
    client:         Client,
    base_url:       String,
    token:          String,
    datasource_uid: String,
}

#[derive(Deserialize)]
    pub struct CreatedDashboard {
        pub uid: String,
        pub url: String,  // e.g. "/d/{uid}/{slug}"
    }

/* ============================================================================================== */
impl GrafanaClient {
    pub fn new(
        client: Client,
        base_url: String,
        token: String,
        datasource_uid: String
    ) -> Self {
        Self { client, base_url, token, datasource_uid }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }

    /* ======================================= Dashboards ======================================= */

    pub async fn create_dashboard(
        &self,
        title: &str,
        table_name: &str, // actual Postgres table name in NocoDB DB
        fields: &[FieldDefinition],
    ) -> Result<CreatedDashboard, AppError> {
        let panels = self.build_panels(table_name, fields);
        let body = json!({
            "dashboard": {
                "uid":           serde_json::Value::Null,
                "title":         title,
                "tags":          ["mosaic-generated"],
                "timezone":      "browser",
                "schemaVersion": 38,
                "version":       0,
                "refresh":       "30s",
                "time":          { "from": "now-7d", "to": "now" },
                "panels":        panels,
            },
            "overwrite": false,
            "message":   "Created by Mosaic orchestrator",
        });

        self.client
            .post(self.url("/api/dashboards/db"))
            .header("Authorization", self.auth())
            .json(&body)
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(
                anyhow::anyhow!("Grafana create_dashboard failed: {e}")
            ))?
            .json::<CreatedDashboard>().await
            .map_err(|e| AppError::Internal(e.into()))
    }

    pub async fn delete_dashboard(&self, uid: &str) -> Result<(), AppError> {
        self.client
            .delete(self.url(&format!("/api/dashboards/uid/{uid}")))
            .header("Authorization", self.auth())
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    /* ====================================== Panel builder ===================================== */

    fn build_panels(&self, table_name: &str, fields: &[FieldDefinition]) -> Value {
        let has_measured_at = fields.iter().any(|f| f.name == "measured_at");
        let time_expr = if has_measured_at {
            "COALESCE(measured_at, created_at)"
        } else {
            "created_at"
        };

        let numeric: Vec<&FieldDefinition> = fields
            .iter()
            .filter(|f| f.field_type == "number")
            .collect();

        let panels: Vec<Value> = numeric.iter().enumerate().map(|(i, field)| {
            let title = match &field.unit {
                Some(u) => format!("{} ({})", field.name, u),
                None => field.name.clone(),
            };

            let sql = format!(
                "SELECT\n  {time} AS time,\n  {col}\nFROM {tbl}\nWHERE $__timeFilter({time})\nORDER BY time",
                time = time_expr,
                col = field.name,
                tbl = table_name
            );

            json!({
                "id":    i + 1,
                "type":  "timeseries",
                "title": title,
                "datasource": { "type": "postgres", "uid": self.datasource_uid },
                "targets": [{
                    "rawSql":      sql,
                    "rawQuery":    true,
                    "format":      "time_series",
                    "refId":       "A",
                    "datasource":  { "type": "postgres", "uid": self.datasource_uid },
                }],
                "fieldConfig": {
                    "defaults": { "custom": { "lineWidth": 2 } },
                    "overrides": []
                },
                "options":  {},
                "gridPos":  {
                    "x": 0,
                    "y": (i as u32) * 8,
                    "w": 24,
                    "h": 8,
                },
            })
        }).collect();

        json!(panels)
    }
}
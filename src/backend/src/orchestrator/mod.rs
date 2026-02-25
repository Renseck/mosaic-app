pub mod grafana_client;
pub mod nocodb_client;
pub mod provisioner;

use std::{io::pipe, sync::Arc};
use sqlx::PgPool;


use crate::db::repos::{
    dashboard_repo::{self, CreateDashboard, DashboardRepo, PgDashboardRepo},
    panel_repo::{CreatePanel, PanelRepo, PgPanelRepo},
    template_repo::{PgTemplateRepo, Template, TemplateRepo},
};
use crate::error::AppError;
use provisioner::{CreateTemplate, Pipeline, Unstarted};
pub use provisioner::CreateTemplate as CreateTemplateInput;
pub use nocodb_client::{NocodbClient};
pub use grafana_client::GrafanaClient;
use uuid::Uuid;

/* ============================================================================================== */
/*                                          Orchestrator                                          */
/* ============================================================================================== */

pub struct Orchestrator {
    pub nocodb:     NocodbClient,
    pub grafana:    GrafanaClient,
    pub pool:       PgPool
}

impl Orchestrator {
    /// Run the full provisioning pipeline:
    /// NocoDB table → form → Grafana dashboard → DB record → portal dashboard.
    pub async fn provision_dataset(
        &self,
        input: CreateTemplateInput,
        user_id: Uuid,
    ) -> Result<Template, AppError> {
        let pipeline = Pipeline::new(input, user_id);

        // Step 1 - NocoDB table + columns
         let pipeline = pipeline.create_table(&self.nocodb).await
            .map_err(|(e, _)| e)?;

        // Step 2 - NocoDB form view (cleanup on failure)
        let pipeline = match pipeline.create_form(&self.nocodb).await {
            Ok(p) => p,
            Err((e, prev)) => {
                tracing::warn!("Form creation failed; cleaning up table '{}': {e}", prev.state.table_id);
                let _ = self.nocodb.delete_table(&prev.state.table_id).await;
                return Err(e);
            }
        };

        // Step 3 - Grafana dashboard (cleanup on fialure)
        let pipeline = match pipeline.create_grafana_dashboard(&self.grafana).await {
            Ok(p) => p,
            Err((e, prev)) => {
                tracing::warn!("Grafana dashboard creation failed; cleaning up NocoDB table: {e}");
                let _ = self.nocodb.delete_table(&prev.state.table_id).await;
                return Err(e);
            }
        };

        // Step 4 - Persist to DB
        let grafana_uid = pipeline.state.grafana_dashboard_uid.clone();
        let grafana_url_path = pipeline.state.grafana_dashboard_url.clone();
        let form_uuid = pipeline.state.form_share_uuid.clone();

        let template = pipeline.register(&self.pool).await?;

        // Step 5 - Auto-create portal dashboard (best-effort; don't fail provisioning)
        if let Err(e) = self.auto_create_portal_dashboard(
            &template, user_id, &grafana_uid, &grafana_url_path, &form_uuid,
        ).await {
            tracing::warn!("Auto-portal-dashboard creation failed (non-fatal): {e}");
        }

        Ok(template)
    }

    /// Best-effort cleanup when a template is deleted.
    pub async fn deprovision_dataset(&self, template: &Template) {
        if let Some(ref table_id) = template.nocodb_table_id {
            if let Err(e) = self.nocodb.delete_table(table_id).await {
                tracing::warn!("Failed to delete NocoDB table '{table_id}': {e}");
            }
        }
        if let Some(ref uid) = template.grafana_dashboard_uid {
            if let Err(e) = self.grafana.delete_dashboard(uid).await {
                tracing::warn!("Failed to delete Grafana dashboard '{uid}': {e}");
            }
        }
    }

    /* ======================================== Internal ======================================== */
    
    async fn auto_create_portal_dashboard(
        &self,
        template:           &Template,
        owner_id:           Uuid,
        grafana_uid:        &str,
        grafana_url:        &str,  // e.g. "/d/{uid}/{slug}"
        nocodb_form_uuid:   &str,
    ) -> Result<(), AppError> {
        let dashboard_repo = PgDashboardRepo { pool: self.pool.clone() };
        let panel_repo = PgPanelRepo { pool: self.pool.clone() };

        let dashboard = dashboard_repo.create(owner_id, CreateDashboard { 
            title: template.name.clone(), 
            slug:       None,  // Auto-generated from title
            icon:       Some("▦".to_string()), 
            sort_order: None, 
            is_shared:  Some(false), 
        }).await?;

        // Grafana full-dashboard panel (kiosk mode — no chrome)
        // Path: /d/{uid}/{slug}?kiosk (strip leading "/d" since we proxy under /proxy/grafana)
        // let grafana_proxy_url = format!("/proxy/grafana{}?kiosk", grafana_url);
        // panel_repo.create(dashboard.id, CreatePanel { 
        //     title: Some(format!("{} - Charts", template.name)), 
        //     panel_type: "grafana_dashboard".to_string(), 
        //     source_url: Some(grafana_proxy_url), 
        //     config:     None, 
        //     grid_x:     0, 
        //     grid_y:     0, 
        //     grid_w:     Some(12), 
        //     grid_h:     Some(8), 
        // }).await?;

        // Individual Grafana panel embeds (one per numeric field)
        let grafana_slug = grafana_url
            .rsplit('/')
            .next()
            .unwrap_or("dashboard");

        let empty_vec = Vec::new();
        let numeric_fields: Vec<_> = template.fields.as_array()
            .unwrap_or(&empty_vec)
            .iter()
            .filter(|f| f.get("field_type").and_then(|v| v.as_str()) == Some("number"))
            .collect();

        for (i, _field) in numeric_fields.iter().enumerate()
        {
            let panel_url = format!(
                "/proxy/grafana/d/{grafana_uid}/{grafana_slug}?viewPanel=panel-{}",
                i + 1
            );
            panel_repo.create(dashboard.id, CreatePanel {
                title:      Some(format!("{} - Panel {}", template.name, i + 1)),
                panel_type: "grafana_panel".to_string(),
                source_url: Some(panel_url),
                config:     None,
                grid_x:     0,
                grid_y:     (i as i32) * 8,
                grid_w:     Some(12),
                grid_h:     Some(8),
            }).await?;
        }

        // NocoDB data-entry form panel
        // let nocodb_form_url = format!("/proxy/nocodb/dashboard/#/nc/form/{}", nocodb_form_uuid);
        // panel_repo.create(dashboard.id, CreatePanel {
        //     title:      Some(format!("{} — Entry Form", template.name)),
        //     panel_type: "nocodb_form".to_string(),
        //     source_url: Some(nocodb_form_url),
        //     config:     None,
        //     grid_x:     0, 
        //     grid_y:     (numeric_fields.len() as i32) * 8, 
        //     grid_w:     Some(12), 
        //     grid_h:     Some(6),
        // }).await?;

        Ok(())
    }
}
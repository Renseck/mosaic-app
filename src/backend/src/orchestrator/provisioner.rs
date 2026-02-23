use axum::Form;
use serde_json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::repos::{
    dashboard_repo::{CreateDashboard, DashboardRepo, PgDashboardRepo},
    panel_repo::{CreatePanel, PanelRepo, PgPanelRepo},
    template_repo::{CreateTemplateRecord, FieldDefinition, PgTemplateRepo, Template, TemplateRepo},
}, proxy::nocodb};
use crate::error::AppError;
use super::nocodb_client::NocodbClient;
use super::grafana_client::GrafanaClient;

/* ============================================================================================== */
/*                                              Input                                             */
/* ============================================================================================== */

#[derive(Debug)]
pub struct CreateTemplate {
    pub name:           String,
    pub description:    Option<String>,
    pub fields:         Vec<FieldDefinition>,
}

/* ============================================================================================== */
/*                                           Type states                                          */
/* ============================================================================================== */

pub struct Unstarted;

pub struct TableReady {
    pub base_id:    String,  // NocoDB base ID
    pub table_id:   String,  // NocoDB table ID
    pub table_name: String,  // Actual Postgres table name
}


pub struct FormReady {
    pub base_id:         String,
    pub table_id:        String,
    pub table_name:      String,
    pub form_view_id:    String,
    pub form_share_uuid: String,   // public UUID for `/nc/form/{uuid}`
}

pub struct GrafanaReady {
    pub table_id:              String,
    pub table_name:            String,
    pub form_view_id:          String,
    pub form_share_uuid:       String,
    pub grafana_dashboard_uid: String,
    pub grafana_dashboard_url: String,
}

/* ============================================================================================== */
/*                                            Pipeline                                            */
/* ============================================================================================== */

pub struct Pipeline<S> {
    pub input:      CreateTemplate,
    pub user_id:    Uuid,
    pub state:      S,
}

impl Pipeline<Unstarted> {
    pub fn new(input: CreateTemplate, user_id: Uuid) -> Self {
        Self { input, user_id, state: Unstarted }
    }

    /// Step 1: Create NocoDB table + columns.
    /// On error the caller gets back ownership of the pipeline for cleanup.
    pub async fn create_table(
        self,
        nocodb: &NocodbClient,
    ) -> Result<Pipeline<TableReady>, (AppError, Pipeline<Unstarted>)> {
        let base_id = match nocodb.get_first_base_id().await {
            Ok(id) => id,
            Err(e) => return Err((e, self)),
        };

        let created = match nocodb.create_table(&base_id, &self.input.name, &self.input.fields).await {
            Ok(t) => t,
            Err(e) => return Err((e, self)),
        };

        Ok(Pipeline {
            state: TableReady { 
                base_id,
                table_id: created.id, 
                table_name: created.table_name 
            },
            input: self.input,
            user_id: self.user_id,
        })
    }
}

/* ============================================================================================== */
impl Pipeline<TableReady> {
    /// Step 2: Create form view and share it.
    pub async fn create_form(
        self,
        nocodb: &NocodbClient,
    ) -> Result<Pipeline<FormReady>, (AppError, Pipeline<TableReady>)> {
        let form_title = format!("{} - Entry Form", self.input.name);
        match nocodb.create_shared_form(&self.state.table_id, &form_title).await {
            Ok((view_id, uuid)) => Ok(Pipeline {
                state: FormReady { 
                    base_id:            self.state.base_id,
                    table_id:           self.state.table_id, 
                    table_name:         self.state.table_name, 
                    form_view_id:       view_id, 
                    form_share_uuid:    uuid, 
                },
                input:      self.input,
                user_id:    self.user_id,
            }),
            Err(e) => Err((e, self)),
        }
    }
}

/* ============================================================================================== */
impl Pipeline<FormReady> {
    /// Step 3: Create Grafana dashboard with one panel per numeric field.
    pub async fn create_grafana_dashboard(
        self,
        grafana: &GrafanaClient,
    ) -> Result<Pipeline<GrafanaReady>, (AppError, Pipeline<FormReady>)> {
        match grafana.create_dashboard(
            &self.input.name,
            &self.state.base_id,
            &self.state.table_name,
            &self.input.fields,
        ).await {
            Ok(created) => Ok(Pipeline { 
                state:  GrafanaReady {
                    table_id:               self.state.table_id,
                    table_name:             self.state.table_name,
                    form_view_id:           self.state.form_view_id,
                    form_share_uuid:        self.state.form_share_uuid,
                    grafana_dashboard_uid:  created.uid,
                    grafana_dashboard_url:  created.url,
                },
                input:      self.input, 
                user_id:    self.user_id, 
                }),
            Err(e) => Err((e, self)),
        }
    }
}

/* ============================================================================================== */
impl Pipeline<GrafanaReady> {
    /// Step 4: Persist template record to the portal DB.
    pub async fn register(self, pool: &PgPool) -> Result<Template, AppError> {
        let repo = PgTemplateRepo { pool: pool.clone() };
        let fields_json = serde_json::to_value(&self.input.fields)
            .map_err(|e| AppError::Internal(e.into()))?;

        repo.create(CreateTemplateRecord { 
            name:                   self.input.name, 
            description:            self.input.description, 
            fields:                 fields_json, 
            created_by:             self.user_id, 
            nocodb_table_id:        Some(self.state.table_id), 
            nocodb_form_id:         Some(self.state.form_share_uuid), 
            grafana_dashboard_uid:  Some(self.state.grafana_dashboard_uid), 
        })
        .await
    }
}
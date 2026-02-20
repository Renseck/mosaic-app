use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/* ============================================================================================== */
/*                                          Domain types                                          */
/* ============================================================================================== */

#[derive(Debug, Clone, Serialize)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub nocodb_table_id: Option<String>,
    pub nocodb_form_id: Option<String>,
    pub grafana_dashboard_uid: Option<String>,
    pub fields: JsonValue,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/* ============================================================================================== */
/*                                        Repository trait                                        */
/* ============================================================================================== */

pub trait TemplateRepo: Send + Sync {}

/* ============================================================================================== */
/*                                     Postgres implementation                                    */
/* ============================================================================================== */

pub struct PgTemplateRepo {
    pub pool: sqlx::PgPool,
}

macro_rules! map_template {
    ($r:expr) => {
        Template {
            id: $r.id,
            name: $r.name,
            description: $r.description,
            nocodb_table_id: $r.nocodb_table_id,
            nocodb_form_id: $r.nocodb_form_id,
            grafana_dashboard_uid: $r.grafana_dashboard_uid,
            fields: $r.fields,
            created_by: $r.created_by,
            created_at: $r.created_at,
            updated_at: $r.updated_at,
        }
    };
}

// TODO: Finish in Phase 7
/* ============================================================================================== */
#[async_trait::async_trait]
impl TemplateRepo for PgTemplateRepo {}
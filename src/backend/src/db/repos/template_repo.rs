use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/* ============================================================================================== */
/*                                          Domain types                                          */
/* ============================================================================================== */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name:       String,            // lowercase alphanumeric + underscore
    pub field_type: String,            // "number" | "text" | "date" | "select"
    pub unit:       Option<String>,    // e.g. "kg", "%", "bpm"
}

#[derive(Debug, Clone, Serialize)]
pub struct Template {
    pub id:                     Uuid,
    pub name:                   String,
    pub description:            Option<String>,
    pub nocodb_table_id:        Option<String>,
    pub nocodb_form_id:         Option<String>,
    pub grafana_dashboard_uid:  Option<String>,
    pub fields:                 JsonValue,
    pub created_by:             Option<Uuid>,
    pub created_at:             DateTime<Utc>,
    pub updated_at:             DateTime<Utc>,
}

/// Used by the orchestrator after external resources are created — all IDs are
/// known before the DB row is inserted.
#[derive(Debug)]
pub struct CreateTemplateRecord {
    pub name:                  String,
    pub description:           Option<String>,
    pub fields:                JsonValue,            // serialised Vec<FieldDefinition>
    pub created_by:            Uuid,
    pub nocodb_table_id:       Option<String>,
    pub nocodb_form_id:        Option<String>,
    pub grafana_dashboard_uid: Option<String>,

}

/* ============================================================================================== */
/*                                        Repository trait                                        */
/* ============================================================================================== */

#[async_trait::async_trait]
pub trait TemplateRepo: Send + Sync {
    async fn list_all(&self) -> Result<Vec<Template>, AppError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Template, AppError>;
    async fn create(&self, record: CreateTemplateRecord) -> Result<Template, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

/* ============================================================================================== */
/*                                     Postgres implementation                                    */
/* ============================================================================================== */

pub struct PgTemplateRepo {
    pub pool: sqlx::PgPool,
}

macro_rules! map_template {
    ($r:expr) => {
        Template {
            id:                     $r.id,
            name:                   $r.name,
            description:            $r.description,
            nocodb_table_id:        $r.nocodb_table_id,
            nocodb_form_id:         $r.nocodb_form_id,
            grafana_dashboard_uid:  $r.grafana_dashboard_uid,
            fields:                 $r.fields,
            created_by:             $r.created_by,
            created_at:             $r.created_at,
            updated_at:             $r.updated_at,
        }
    };
}

/* ============================================================================================== */
#[async_trait::async_trait]
impl TemplateRepo for PgTemplateRepo {
    async fn list_all(&self) -> Result<Vec<Template>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, description, nocodb_table_id, nocodb_form_id, 
                   grafana_dashboard_uid, fields as "fields!: JsonValue",
                   created_by, created_at, updated_at
            FROM portal.dataset_templates
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| map_template!(r)).collect())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Template, AppError> {
        sqlx::query!(
            r#"
            SELECT id, name, description, nocodb_table_id, nocodb_form_id, 
                   grafana_dashboard_uid, fields as "fields!: JsonValue",
                   created_by, created_at, updated_at
            FROM portal.dataset_templates
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_template!(r))
        .ok_or_else(|| AppError::NotFound(format!("template '{id}' not found")))
    }

    async fn create(&self, record: CreateTemplateRecord) -> Result<Template, AppError> {
        sqlx::query!(
            r#"
            INSERT INTO portal.dataset_templates
                (name, description, fields, created_by,
                 nocodb_table_id, nocodb_form_id, grafana_dashboard_uid)
            VALUES ($1, $2, $3::jsonb, $4, $5, $6, $7)
            RETURNING id, name, description, nocodb_table_id, nocodb_form_id,
                      grafana_dashboard_uid, fields as "fields!: JsonValue",
                      created_by, created_at, updated_at
            "#,
            record.name,
            record.description,
            record.fields,
            record.created_by,
            record.nocodb_table_id,
            record.nocodb_form_id,
            record.grafana_dashboard_uid,
        )
        .fetch_one(&self.pool)
        .await
        .map(|r| map_template!(r))
        .map_err(AppError::Database)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query!(
            "DELETE FROM portal.dataset_templates WHERE id = $1", id
        )
        .execute(&self.pool)
        .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("template '{id}' not found")))
        }
        Ok(())
    }
}
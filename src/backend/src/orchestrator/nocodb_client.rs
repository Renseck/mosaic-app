use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::db::repos::template_repo::FieldDefinition;
use crate::error::AppError;

/* ============================================================================================== */
/*                                       Postgres connection                                      */
/* ============================================================================================== */

/// Connection details NocoDB uses to reach the external Postgres data source.
/// These are Docker-internal coordinates (hostname = service name).
pub struct NocoPgConnection {
    pub host:     String,
    pub port:     u16,
    pub user:     String,
    pub password: String,
    pub database: String,
}

/* ============================================================================================== */
/*                                          NocoDB Client                                         */
/* ============================================================================================== */

pub struct NocodbClient {
    client:     Client,
    base_url:   String,
    token:      String,
    pg_conn:    NocoPgConnection,
}

#[derive(Deserialize)]
pub struct CreatedTable {
    pub id:         String,   // NocoDB table ID (md_xxx)
    pub table_name: String,   // actual Postgres table name (nc_p_xxx_name)
}

#[derive(Deserialize)]
struct CreatedView { 
    id: String 
}

/* ============================================================================================== */
impl NocodbClient {
    pub fn new(
        client: Client,
        base_url: String,
        token: String,
        pg_conn: NocoPgConnection,
    ) -> Self {
        Self { client, base_url, token, pg_conn }
    }

    fn url (&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn auth(&self) -> (&str, &str) {
        ("xc-token", &self.token)
    }

    /* ========================================== Bases ========================================= */

    /// Find or create an external-Postgres-backed base named "Mosaic".
    /// This base is where all user dataset tables live. The method is
    /// idempotent: on subsequent calls it returns the existing base ID.
    pub async fn ensure_mosaic_base(&self) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct Base { id: String, title: String }
        #[derive(Deserialize)]
        struct ListResponse { list: Vec<Base> }

        // 1. Check if the "Mosaic" base already exists
        let resp: ListResponse = self.client
            .get(self.url("/api/v2/meta/bases"))
            .header(self.auth().0, self.auth().1)
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(e.into()))?
            .json().await
            .map_err(|e| AppError::Internal(e.into()))?;

        if let Some(base) = resp.list.into_iter().find(|b| b.title == "Mosaic") {
            tracing::info!("Found existing Mosaic base: {}", base.id);
            return Ok(base.id);
        }

        // 2. Create a new base with an external Postgres source
        #[derive(Deserialize)]
        struct CreatedBase { id: String }

        let conn = &self.pg_conn;
        let created: CreatedBase = self.client
            .post(self.url("/api/v2/meta/bases"))
            .header(self.auth().0, self.auth().1)
            .json(&json!({
                "title": "Mosaic",
                "sources": [{
                    "type":   "pg",
                    "config": {
                        "client":     "pg",
                        "connection": {
                            "host":     conn.host,
                            "port":     conn.port,
                            "user":     conn.user,
                            "password": conn.password,
                            "database": conn.database,
                        },
                    },
                }],
            }))
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(
                anyhow::anyhow!("NocoDB create base failed: {e}")
            ))?
            .json().await
            .map_err(|e| AppError::Internal(e.into()))?;

        tracing::info!("Created Mosaic base: {}", created.id);
        Ok(created.id)
    }

    
    /* ========================================================================================== */
    /// Create a table with all columns in a single request, as required by NocoDB v2 API.
    pub async fn create_table(
        &self,
        base_id: &str,
        title: &str,
        fields: &[FieldDefinition],
    ) -> Result<CreatedTable, AppError> {
        let columns: Vec<serde_json::Value> = fields.iter().map(|field| {
            let uidt = match field.field_type.as_str() {
                "number" => "Number",
                "date"   => "Date",
                "select" => "SingleSelect",
                _        => "SingleLineText",
            };
            json!({ "title": field.name, "uidt": uidt })
        }).collect();

        let resp = self.client
            .post(self.url(&format!("/api/v2/meta/bases/{base_id}/tables")))
            .header(self.auth().0, self.auth().1)
            .json(&json!({
                "title":   title,
                "columns": columns,
            }))
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(
                anyhow::anyhow!("NocoDB create_table failed: {e}")
            ))?;

        // Log the raw response to diagnose field naming
        let body: serde_json::Value = resp.json().await
            .map_err(|e| AppError::Internal(e.into()))?;
        tracing::info!("NocoDB create_table response: {}", body);

        let created: CreatedTable = serde_json::from_value(body)
            .map_err(|e| AppError::Internal(
                anyhow::anyhow!("NocoDB create_table response parse failed: {e}")
            ))?;

        Ok(created)
    }

    /* ========================================================================================== */
    /// Delete a table by its NocoDB table ID.
    pub async fn delete_table(&self, table_id: &str) -> Result<(), AppError> {
        self.client
            .delete(self.url(&format!("/api/v2/meta/tables/{table_id}")))
            .header(self.auth().0, self.auth().1)
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    /* ========================================= Columns ======================================== */

    /// This method may be useful to keep around for updating tables later.
    pub async fn add_columns(
        &self,
        table_id: &str,
        fields: &[FieldDefinition],
    ) -> Result<(), AppError> {
        for field in fields {
            let uidt = match field.field_type.as_str() {
                "number" => "Number",
                "date"   => "Date",
                "select" => "SingleSelect",
                _        => "SingleLineText",
            };
            self.client
                .post(self.url(&format!("/api/v2/meta/tables/{table_id}/fields")))
                .header(self.auth().0, self.auth().1)
                .json(&json!({ "title": field.name, "uidt": uidt }))
                .send().await
                .map_err(|e| AppError::Internal(e.into()))?
                .error_for_status()
                .map_err(|e| AppError::Internal(
                    anyhow::anyhow!("NocoDB create_column '{}' failed: {e}", field.name)
                ))?;
        }
        Ok(())
    }

    /* ======================================= Form views ======================================= */

    /// Create a form view and return a publicly shareable form URL segment.
    /// Returns `(view_id, share_uuid)` — embed via `/proxy/nocodb/nc/form/{uuid}`.
    pub async fn create_shared_form(
        &self,
        table_id: &str,
        title: &str,
    ) -> Result<(String, String), AppError> {
        // 1. Create form view (type = "form")
        let view: CreatedView = self.client
            .post(self.url(&format!("/api/v2/meta/tables/{table_id}/views")))
            .header(self.auth().0, self.auth().1)
            .json(&json!({ "title": title, "type": "form" }))
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(
                anyhow::anyhow!("NocoDB create_view failed: {e}")
            ))?
            .json().await
            .map_err(|e| AppError::Internal(e.into()))?;

        // 2. Enable sharing → get public UUID
        #[derive(Deserialize)]
        struct ShareResponse { uuid: String }
        let share: ShareResponse = self.client
            .post(self.url(&format!("/api/v2/meta/views/{}/share", view.id)))
            .header(self.auth().0, self.auth().1)
            .json(&json!({}))
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(
                anyhow::anyhow!("NocoDB share_view failed: {e}")
            ))?
            .json().await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok((view.id, share.uuid))
    }
}
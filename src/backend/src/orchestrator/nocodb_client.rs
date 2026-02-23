use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::db::repos::template_repo::FieldDefinition;
use crate::error::AppError;

/* ============================================================================================== */
/*                                          NocoDB Client                                         */
/* ============================================================================================== */

pub struct NocodbClient {
    client:     Client,
    base_url:   String,
    token:      String,
}

#[derive(Deserialize)]
pub struct CreatedTable {
    pub id:         String,   // NocoDB table ID (md_xxx)
    pub table_name: String,   // actual Postgres table name (nc_p_xxx_name)
}

#[derive(Deserialize)]
struct CreatedFormView { 
    id:     String,
    title:  String,
}

/* ============================================================================================== */
impl NocodbClient {
    pub fn new(
        client: Client,
        base_url: String,
        token: String,
    ) -> Self {
        Self { client, base_url, token }
    }

    fn url (&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn auth(&self) -> (&str, &str) {
        ("xc-token", &self.token)
    }

    /* ========================================== Bases ========================================= */

    /// Discover the first NocoDB base. NocoDB stores both meta and data in the
    /// same Postgres database (configured via `NC_DB`), so the default base
    /// already creates real Postgres tables — no external source needed.
    pub async fn get_first_base_id(&self) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct Base { id: String }
        #[derive(Deserialize)]
        struct Response { list: Vec<Base> }

        let resp: Response = self.client
            .get(self.url("/api/v2/meta/bases"))
            .header(self.auth().0, self.auth().1)
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(e.into()))?
            .json().await
            .map_err(|e| AppError::Internal(e.into()))?;

        resp.list.into_iter().next()
            .map(|b| b.id)
            .ok_or_else(|| AppError::Internal(
                anyhow::anyhow!("NocoDB has no bases — initialize NocoDB first")
            ))
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

        // Wait for the table to become queryable in NocoDB's meta catalog.
        // External Postgres sources may not be immediately consistent.
        self.wait_for_table(&created.id).await?;

        Ok(created)
    }

    /* ========================================================================================== */
    /// Poll NocoDB until the table is findable, or fail after timeout.
    /// This guards against race conditions with external data sources.
    async fn wait_for_table(&self, table_id: &str) -> Result<(), AppError> {
        let max_attempts = 10;
        let delay = std::time::Duration::from_millis(500);

        for attempt in 1..=max_attempts {
            let resp = self.client
                .get(self.url(&format!("/api/v2/meta/tables/{table_id}/views")))
                .header(self.auth().0, self.auth().1)
                .send().await
                .map_err(|e| AppError::Internal(e.into()))?;

            if resp.status().is_success() {
                tracing::info!(
                    "Table {} ready after {} attempt(s)",
                    table_id,
                    attempt
                );
                return Ok(());
            }

            tracing::warn!(
                "Table {} not yet ready (attempt {}/{}), retrying…",
                table_id,
                attempt,
                max_attempts
            );
            tokio::time::sleep(delay).await;
        }

        Err(AppError::Internal(anyhow::anyhow!(
            "Table {} not queryable after {} attempts — NocoDB may not have \
             registered it in its meta catalog",
            table_id,
            max_attempts
        )))
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
        //    POST /api/v2/meta/tables/{tableId}/forms
        //    Body: { "title": "...", "type": 1 }
        let view: CreatedFormView = self.client
            .post(self.url(&format!("/api/v2/meta/tables/{table_id}/forms")))
            .header(self.auth().0, self.auth().1)
            .json(&json!({
                "title": title,
                "type":  1
            }))
            .send().await
            .map_err(|e| AppError::Internal(e.into()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(
                anyhow::anyhow!("NocoDB create_form failed: {e}")
            ))?
            .json().await
            .map_err(|e| AppError::Internal(e.into()))?;

        tracing::info!("Created NocoDB form view '{}' (id: {})", title, view.id);

        // 2. Enable sharing → get public UUID
        //    POST /api/v2/meta/views/{viewId}/share
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

        tracing::info!("Shared form view {} → uuid {}", view.id, share.uuid);

        Ok((view.id, share.uuid))
    }
}
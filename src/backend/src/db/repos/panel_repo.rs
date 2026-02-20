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
pub struct Panel {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub title: Option<String>,
    pub panel_type: String,
    pub source_url: Option<String>,
    pub config: JsonValue,
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_w: i32,
    pub grid_h: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePanel {
    pub title: Option<String>,
    pub panel_type: String,
    pub source_url: Option<String>,
    pub config: Option<JsonValue>,
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_w: Option<i32>,
    pub grid_h: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePanel {
    pub title: Option<String>,
    pub panel_type: Option<String>,
    pub source_url: Option<String>,
    pub config: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct GridPosition {
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_w: i32,
    pub grid_h: i32,
}

#[derive(Debug, Deserialize)]
pub struct BatchPositionUpdate {
    pub id: Uuid,
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_w: i32,
    pub grid_h: i32,
}

/* ============================================================================================== */
/*                                        Repository trait                                        */
/* ============================================================================================== */

#[async_trait::async_trait]
pub trait PanelRepo: Send + Sync {
    async fn list_for_dashboard(&self, dashboard_id: Uuid) -> Result<Vec<Panel>, AppError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Panel, AppError>;
    async fn create(&self, dashboard_id: Uuid, input: CreatePanel) -> Result<Panel, AppError>;
    async fn update(&self, id: Uuid, input: UpdatePanel) -> Result<Panel, AppError>;
    async fn update_position(&self, id: Uuid, pos: GridPosition) -> Result<Panel, AppError>;
    async fn batch_update_positions(&self, updates: Vec<BatchPositionUpdate>) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

/* ============================================================================================== */
/*                                     Postgres implementation                                    */
/* ============================================================================================== */

pub struct PgPanelRepo {
    pub pool: PgPool,
}

macro_rules! map_panel {
    ($r:expr) => {
        Panel {
            id: $r.id,
            dashboard_id: $r.dashboard_id,
            title: $r.title,
            panel_type: $r.panel_type,
            source_url: $r.source_url,
            config: $r.config,
            grid_x: $r.grid_x,
            grid_y: $r.grid_y,
            grid_w: $r.grid_w,
            grid_h: $r.grid_h,
            created_at: $r.created_at,
            updated_at: $r.updated_at,
        }
    };
}

/* ============================================================================================== */
#[async_trait::async_trait]
impl PanelRepo for PgPanelRepo {
    async fn list_for_dashboard(&self, dashboard_id: Uuid) -> Result<Vec<Panel>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, dashboard_id, title, panel_type, source_url,
                   config as "config!: JsonValue",
                   grid_x, grid_y, grid_w, grid_h, created_at, updated_at
            FROM portal.panels
            WHERE dashboard_id = $1
            ORDER BY grid_y ASC, grid_x ASC
            "#,
            dashboard_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| map_panel!(r)).collect())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Panel, AppError> {
        sqlx::query!(
            r#"
            SELECT id, dashboard_id, title, panel_type, source_url,
                   config as "config!: JsonValue",
                   grid_x, grid_y, grid_w, grid_h, created_at, updated_at
            FROM portal.panels
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_panel!(r))
        .ok_or_else(|| AppError::NotFound(format!("panel '{id}' not found")))
    }

    async fn create(&self, dashboard_id: Uuid, input: CreatePanel) -> Result<Panel, AppError> {
        let config = input.config.unwrap_or(serde_json::json!({}));

        sqlx::query!(
            r#"
            INSERT INTO portal.panels
                (dashboard_id, title, panel_type, source_url, config, grid_x, grid_y, grid_w, grid_h)
            VALUES ($1, $2, $3, $4, $5::jsonb, $6, $7, $8, $9)
            RETURNING id, dashboard_id, title, panel_type, source_url,
                      config as "config!: JsonValue",
                      grid_x, grid_y, grid_w, grid_h, created_at, updated_at
            "#,
            dashboard_id,
            input.title,
            input.panel_type,
            input.source_url,
            config,
            input.grid_x,
            input.grid_y,
            input.grid_w.unwrap_or(6),
            input.grid_h.unwrap_or(4),
        )
        .fetch_one(&self.pool)
        .await
        .map(|r| map_panel!(r))
        .map_err(AppError::Database)
    }

    async fn update(&self, id: Uuid, input: UpdatePanel) -> Result<Panel, AppError> {
        sqlx::query!(
            r#"
            UPDATE portal.panels
            SET title      = COALESCE($2, title),
                panel_type = COALESCE($3, panel_type),
                source_url = COALESCE($4, source_url),
                config     = COALESCE($5::jsonb, config),
                updated_at = now()
            WHERE id = $1
            RETURNING id, dashboard_id, title, panel_type, source_url,
                      config as "config!: JsonValue",
                      grid_x, grid_y, grid_w, grid_h, created_at, updated_at
            "#,
            id,
            input.title,
            input.panel_type,
            input.source_url,
            input.config,
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_panel!(r))
        .ok_or_else(|| AppError::NotFound(format!("panel '{id}' not found")))
    }

    async fn update_position(&self, id: Uuid, pos: GridPosition) -> Result<Panel, AppError> {
        sqlx::query!(
            r#"
            UPDATE portal.panels
            SET grid_x     = $2,
                grid_y     = $3,
                grid_w     = $4,
                grid_h     = $5,
                updated_at = now()
            WHERE id = $1
            RETURNING id, dashboard_id, title, panel_type, source_url,
                      config as "config!: JsonValue",
                      grid_x, grid_y, grid_w, grid_h, created_at, updated_at
            "#,
            id,
            pos.grid_x,
            pos.grid_y,
            pos.grid_w,
            pos.grid_h,
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_panel!(r))
        .ok_or_else(|| AppError::NotFound(format!("panel '{id}' not found")))
    }

    async fn batch_update_positions(&self, updates: Vec<BatchPositionUpdate>) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        for u in updates {
            sqlx::query!(
                "UPDATE portal.panels
                SET grid_x = $2, grid_y = $3, grid_w = $4, grid_h = $5, updated_at = now()
                WHERE id = $1",
                u.id, u.grid_x, u.grid_y, u.grid_w, u.grid_h
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM portal.panels WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("panel '{id}' not found")));
        }
        Ok(())
    }
}
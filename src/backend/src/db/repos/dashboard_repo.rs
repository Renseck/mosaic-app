use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/* ============================================================================================== */
/*                                          Domain types                                          */
/* ============================================================================================== */

#[derive(Debug, Clone, Serialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub owner_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_shared: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDashboard {
    pub title: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_shared: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDashboard {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_shared: Option<bool>,
}

/* ============================================================================================== */
/*                                        Repository trait                                        */
/* ============================================================================================== */

#[async_trait::async_trait]
pub trait DashboardRepo: Send + Sync {
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Dashboard>, AppError>;
    async fn get_by_slug(&self, slug: &str) -> Result<Dashboard, AppError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Dashboard, AppError>;
    async fn create(&self, owner_id: Uuid, input: CreateDashboard) -> Result<Dashboard, AppError>;
    async fn update(&self, id: Uuid, input: UpdateDashboard) -> Result<Dashboard, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

/* ============================================================================================== */
/*                                     Postgres implementation                                    */
/* ============================================================================================== */

pub struct PgDashboardRepo {
    pub pool: PgPool,
}

/// Converts a title into a URL-friendly slug (ASCII only).
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

macro_rules! map_dashboard {
    ($r:expr) => {
        Dashboard {
            id: $r.id,
            owner_id: $r.owner_id,
            title: $r.title,
            slug: $r.slug,
            icon: $r.icon,
            sort_order: $r.sort_order,
            is_shared: $r.is_shared,
            created_at: $r.created_at,
            updated_at: $r.updated_at,
        }
    };
}

/* ============================================================================================== */
#[async_trait::async_trait]
impl DashboardRepo for PgDashboardRepo {
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Dashboard>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, owner_id, title, slug, icon, sort_order, is_shared, created_at, updated_at
            FROM portal.dashboards
            WHERE owner_id = $1 OR is_shared = true
            ORDER BY sort_order ASC, title ASC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| map_dashboard!(r)).collect())
    }
    
    async fn get_by_slug(&self, slug: &str) -> Result<Dashboard, AppError> {
        sqlx::query!(
            r#"
            SELECT id, owner_id, title, slug, icon, sort_order, is_shared, created_at, updated_at
            FROM portal.dashboards WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_dashboard!(r))
        .ok_or_else(|| AppError::NotFound(format!("dashboard '{slug}' not found")))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Dashboard, AppError> {
        sqlx::query!(
            r#"
            SELECT id, owner_id, title, slug, icon, sort_order, is_shared, created_at, updated_at
            FROM portal.dashboards WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_dashboard!(r))
        .ok_or_else(|| AppError::NotFound(format!("dashboard '{id}' not found")))
    }

    async fn create(&self, owner_id: Uuid, input: CreateDashboard) -> Result<Dashboard, AppError> {
        let slug = input
            .slug
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| slugify(&input.title));

        sqlx::query!(
            r#"
            INSERT INTO portal.dashboards (owner_id, title, slug, icon ,sort_order, is_shared)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, owner_id, title, slug, icon, sort_order, is_shared, created_at, updated_at
            "#,
            owner_id,
            input.title,
            slug,
            input.icon,
            input.sort_order.unwrap_or(0),
            input.is_shared.unwrap_or(false),
        )
        .fetch_one(&self.pool)
        .await
        .map(|r| map_dashboard!(r))
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) =e {
                if db_err.constraint() == Some("dashboards_slug_key") {
                    return AppError::Validation(format!("slug '{slug}' is already taken"));
                }
            }
            AppError::Database(e)
        })
    }

    async fn update(&self, id: Uuid, input: UpdateDashboard) -> Result<Dashboard, AppError> {
        sqlx::query!(
            r#"
            UPDATE portal.dashboards
            SET title      = COALESCE($2, title),
                slug       = COALESCE($3, slug),
                icon       = COALESCE($4, icon),
                sort_order = COALESCE($5, sort_order),
                is_shared  = COALESCE($6, is_shared),
                updated_at = now() 
            WHERE id = $1
            RETURNING id, owner_id, title, slug, icon, sort_order, is_shared, created_at, updated_at
            "#,
            id,
            input.title,
            input.slug,
            input.icon,
            input.sort_order,
            input.is_shared,
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_dashboard!(r))
        .ok_or_else(|| AppError::NotFound(format!("dashboard '{id}' not found")))
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM portal.dashboards
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("dashboard '{id}' not found")));
        }
        Ok(())
    }
}
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/* ============================================================================================== */
/*                                          Domain types                                          */
/* ============================================================================================== */

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/* ============================================================================================== */
/*                                        Repository trait                                        */
/* ============================================================================================== */

#[async_trait::async_trait]
pub trait UserRepo: Send + Sync {
    async fn list(&self) -> Result<Vec<User>, AppError>;
    async fn update_role(&self, id: Uuid, role: &str) -> Result<User, AppError>;
}

/* ============================================================================================== */
/*                                     Postgres implementation                                    */
/* ============================================================================================== */

pub struct PgUserRepo {
    pub pool: PgPool,
}

macro_rules! map_user {
    ($r:expr) => {
        User {
            id: $r.id,
            username: $r.username,
            email: $r.email,
            role: $r.role,
            created_at: $r.created_at,
            updated_at: $r.updated_at,
        }
    };
}

/* ============================================================================================== */
#[async_trait::async_trait]
impl UserRepo for PgUserRepo {
    async fn list(&self) -> Result<Vec<User>, AppError> {
        let rows = sqlx::query!(
            "SELECT id, username, email, role, created_at, updated_at
            FROM portal.users 
            ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| map_user!(r))
            .collect())
    }

    async fn update_role(&self, id: Uuid, role: &str) -> Result<User, AppError> {
        sqlx::query!(
            r#"
            UPDATE portal.users
            SET role       = $2,
                updated_at = now()
            WHERE id = $1
            RETURNING id, username, email, role, created_at, updated_at
            "#,
            id,
            role
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| map_user!(r))
        .ok_or_else(|| AppError::NotFound(format!("user '{id}' not found")))
    }
}
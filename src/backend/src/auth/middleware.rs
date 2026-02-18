use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::session;
use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Editor => write!(f, "editor"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

impl TryFrom<&str> for Role {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "admin" => Ok(Role::Admin),
            "editor" => Ok(Role::Editor),
            "viewer" => Ok(Role::Viewer),
            _ => Err(AppError::Validation(format!("unknown role: {s}"))),
        }
    }
}

/* ============================================================================================== */
/// Axum extractor that validates the session cookie and returns the authenticated user.
/// Handlers add this to their signature to require authentication.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub username: String,
    pub role: Role
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);
        let token = session::extract_cookie(&parts.headers).ok_or(AppError::Unauthorized)?;
        let user = session::validate_session(&pool, &token).await?;
        let role = Role::try_from(user.role.as_str())?;
        Ok(AuthenticatedUser {
            user_id: user.user_id,
            username: user.username,
            role,
        })
    }
}

/* ============================================================================================== */
/// Extractor that additionally requires the Admin role, rejecting others with 403.
pub struct RequireAdmin(pub AuthenticatedUser);

impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        if user.role != Role::Admin {
            return Err(AppError::Forbidden);
        }
        Ok(RequireAdmin(user))
    }
}
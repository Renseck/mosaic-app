use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

/* ============================================================================================== */
impl Role {
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

/* ============================================================================================== */
impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Editor => write!(f, "editor"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

/* ============================================================================================== */
/// Returned by GET /api/auth/me and POST /api/auth/login
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: Role,
}
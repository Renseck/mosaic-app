use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", axum::routing::get(health))
        .nest("/api/auth", auth_routes())
        .merge(crate::proxy::router())
        .with_state(state)
}

fn auth_routes() -> Router<AppState> {
    use axum::routing::post;
    use crate::auth::handlers::{login, logout, me, register};

    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
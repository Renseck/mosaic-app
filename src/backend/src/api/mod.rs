use axum::{Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new().route("/api/health", axum::routing::get(health))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
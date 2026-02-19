use axum::{routing::get, http::Request, response::Response, Json, Router};
use tower_http::trace::TraceLayer;
use tracing::{Span, Level};
use std::time::Duration;
use serde_json::{json, Value};

use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", axum::routing::get(health))
        .nest("/api/auth", auth_routes())
        .merge(crate::proxy::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let client_ip = request
                        .headers()
                        .get("x-forwarded-for")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string())
                        .or_else(|| {
                            request.extensions()
                                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                                .map(|ci| ci.0.to_string())
                        })
                        .unwrap_or_else(|| "unknown".to_string());
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        client_ip = %client_ip,
                    )
                })
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    tracing::info!(
                        status = %response.status().as_u16(),
                        latency_ms = %latency.as_millis(),
                        "response"
                    );
                })
        )
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
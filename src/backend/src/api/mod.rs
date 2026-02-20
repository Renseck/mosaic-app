mod dashboards;
mod panels;
mod templates;
mod users;

use axum::{
    http::Request,
    response::Response,
    routing::{get, put},
    Json, Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing::{Level, Span};

use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", axum::routing::get(health))
        .nest("/api/auth", auth_routes())
        .nest("/api/dashboards", dashboard_routes())
        .nest("/api/panels", panel_routes())
        .nest("/api/users", user_routes())
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
                }),
        )
        .with_state(state)
}

/* ============================================================================================== */
/*                                          Nested routes                                         */
/* ============================================================================================== */

fn auth_routes() -> Router<AppState> {
    use axum::routing::post;
    use crate::auth::handlers::{login, logout, me, register};

    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

/* ============================================================================================== */
fn dashboard_routes() -> Router<AppState> {
    use crate::api::{
        dashboards::{create_dashboard, delete_dashboard, get_dashboard, list_dashboards, update_dashboard},
        panels::{create_panel, list_panels},
    };

    Router::new()
        .route("/", get(list_dashboards).post(create_dashboard))
        // GET uses slug, PUT/DELETE use UUID - both map to the same path segment.
        .route("/{id}", get(get_dashboard).put(update_dashboard).delete(delete_dashboard))
        .route("/{dashboard_id}/panels", get(list_panels).post(create_panel))
}

/* ============================================================================================== */
fn panel_routes() -> Router<AppState> {
    use crate::api::panels::{
        batch_update_positions, delete_panel, update_panel, update_position,
    };
    
    Router::new()
        // Literal segement - Axum routes this before the parameterised /{id} routes.
        .route("/batch-position", put(batch_update_positions))
        .route("/{id}", put(update_panel).delete(delete_panel))
        .route("/{id}/position", put(update_position))
}

/* ============================================================================================== */
fn user_routes() -> Router<AppState> {
    use crate::api::users::{list_users, update_user_role};

    Router::new()
        .route("/", get(list_users))
        .route("/{id}/role", put(update_user_role))
}

/* ============================================================================================== */
/*                                         Health endpoint                                        */
/* ============================================================================================== */

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
mod api;
mod auth;
mod config;
mod db;
mod error;
mod orchestrator;
mod proxy;
mod spa;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::FromRef;
use config::AppConfig;
use db::pool::create_pool;
use db::repos::{
    dashboard_repo::{DashboardRepo, PgDashboardRepo},
    panel_repo::{PanelRepo, PgPanelRepo},
    user_repo::{PgUserRepo, UserRepo},
};
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

use crate::db::repos::{PgTemplateRepo, TemplateRepo};
use crate::orchestrator::{GrafanaClient, NocodbClient, Orchestrator};

/// Shared application state injected into all handlers via Axum's `State` extractors.
#[derive(Clone)]
pub struct AppState {
    pub pool:           PgPool,
    pub config:         AppConfig,
    pub http_client:    reqwest::Client,
    pub dashboards:     Arc<dyn DashboardRepo>,
    pub panels:         Arc<dyn PanelRepo>,
    pub users:          Arc<dyn UserRepo>,
    pub templates:      Arc<dyn TemplateRepo>,
    pub orchestrator:   Arc<Orchestrator>,
}

/// Allows extractors (e.g. `AuthenticatedUser`) to pull the pool directly from state
/// without needing to import `AppState`.
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

/* ============================================================================================== */
#[tokio::main]
async fn main() {
    // Load .env file from src/ directory (two levels up from the crate root)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let env_path = std::path::Path::new(manifest_dir)
        .parent()                   // src/backend/ → src/
        .expect("parent of CARGO_MANIFEST_DIR must exist");

    // Try src/.env first, then fall back to repo root .env
    let src_env = env_path.join(".env");
    let root_env = env_path
        .parent()
        .map(|p| p.join(".env"))
        .unwrap_or_default();

    if src_env.is_file() {
        dotenvy::from_path(&src_env).ok();
    } else if root_env.is_file() {
        dotenvy::from_path(&root_env).ok();
    } else {
        // Last resort: default behaviour (cwd)
        dotenvy::dotenv().ok();
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = AppConfig::from_env().expect("Failed to load configuration");
    tracing::info!("Starting mosaic-app on {}", config.bind_address);

    // Create database pool
    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Connected to database");

    // Run migrations
    sqlx::migrate!("src/db/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Migrations applied");

    // Build state
    let http_client = reqwest::Client::builder()
        .build()
        .expect("Failed to build HTTP client");

    let nocodb = NocodbClient::new(
        http_client.clone(),
        config.nocodb_internal_url.clone(),
        config.nocodb_api_token.clone(),
    );
    let grafana = GrafanaClient::new(
        http_client.clone(),
        config.grafana_internal_url.clone(),
        config.grafana_service_account_token.clone(),
        config.grafana_datasource_uid.clone(),
    );
    let orchestrator = Arc::new(Orchestrator { nocodb, grafana, pool: pool.clone() });

    let bind_address = config.bind_address.clone();
    let state = AppState {
        pool:         pool.clone(),
        config,
        http_client,
        dashboards:   Arc::new(PgDashboardRepo { pool: pool.clone() }),
        panels:       Arc::new(PgPanelRepo     { pool: pool.clone() }),
        users:        Arc::new(PgUserRepo       { pool: pool.clone() }),
        templates:    Arc::new(PgTemplateRepo   { pool: pool.clone() }),
        orchestrator,
    };

    // Build router
    let app = api::router(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    // Start server
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("Failed to bind address");
    tracing::info!("Listening on {bind_address}");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
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

/// Shared application state injected into all handlers via Axum's `State` extractors.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub dashboards: Arc<dyn DashboardRepo>,
    pub panels: Arc<dyn PanelRepo>,
    pub users: Arc<dyn UserRepo>,
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
    // Load .env file (look in src/ directory and parent)
    dotenvy::dotenv().ok();

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

    let dashboards: Arc<dyn DashboardRepo> = Arc::new(PgDashboardRepo { pool: pool.clone() });
    let panels: Arc<dyn PanelRepo> = Arc::new(PgPanelRepo { pool: pool.clone() });
    let users: Arc<dyn UserRepo> = Arc::new(PgUserRepo { pool: pool.clone() });
    
    let bind_address = config.bind_address.clone();
    let state = AppState {
        pool, 
        config,
        http_client,
        dashboards,
        panels,
        users,   
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
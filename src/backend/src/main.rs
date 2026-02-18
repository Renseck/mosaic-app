mod api;
mod config;
mod db;
mod error;

use config::AppConfig;
use db::pool::create_pool;
use tracing_subscriber::EnvFilter;

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

    // Build router
    let app = api::router();

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .expect("Failed to bind address");
    tracing::info!("Listening on {}", config.bind_address);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
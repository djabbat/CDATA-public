use axum::{Router, Server};
use std::net::SocketAddr;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use cdata_backend::config::Config;
use cdata_backend::db::Database;
use cdata_backend::routes::app_router;
use cdata_backend::error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "cdata_backend=debug,tower_http=debug,axum=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting CDATA backend v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::from_env()
        .map_err(|e| AppError::Configuration(e.to_string()))?;
    info!("Configuration loaded: environment={}", config.environment);

    // Initialize database connection pool
    let db = Database::connect(&config.database_url)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    // Run pending migrations
    db.run_migrations()
        .await
        .map_err(|e| AppError::Migration(e.to_string()))?;
    info!("Database migrations completed");

    // Build application with routes
    let app = app_router(db);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Server listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::Server(e.to_string()))?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutdown signal received");
}
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{info, error};
use std::time::Duration;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        info!("Connecting to database...");
        
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;
        
        info!("Database connection established");
        Ok(Self { pool })
    }
    
    pub async fn run_migrations(&self) -> Result<(), sqlx::migrate::MigrateError> {
        info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
    }
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// Implement Into<PgPool> for easy use with Axum
impl From<Database> for PgPool {
    fn from(db: Database) -> Self {
        db.pool
    }
}
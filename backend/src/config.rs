use serde::Deserialize;
use config::{Config as ConfigBuilder, Environment, File};
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub environment: String,
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        let config = ConfigBuilder::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("CDATA").separator("__"))
            .build()?;
        
        config.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            port: 3003,
            database_url: "postgres://cn:cn@localhost/cdata_db".to_string(),
            log_level: "debug".to_string(),
        }
    }
}
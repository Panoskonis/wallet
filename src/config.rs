use std::env;

/// Application configuration loaded from environment variables
/// This struct holds all configuration values needed by the application
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL database connection URL
    pub database_url: String,
    /// Server port to listen on
    pub port: u16,
    /// Server host address to bind to
    pub host: String,
    /// Logging level (e.g., "debug", "info", "warn")
    pub rust_log: String,
}

impl Config {
    /// Load configuration from environment variables
    /// Uses dotenv to load from .env file if present, then falls back to system env vars
    ///
    /// # Errors
    /// Returns an error if required environment variables are missing or invalid
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env file if it exists (doesn't error if file doesn't exist)
        dotenv::dotenv().ok();

        // Extract environment variables with defaults where appropriate
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable is required"))?;

        // Parse port with a default value
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| anyhow::anyhow!("Invalid PORT value: {}", e))?;

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            database_url,
            port,
            host,
            rust_log,
        })
    }
}

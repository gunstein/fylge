use std::net::SocketAddr;

/// Server configuration from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub database_url: String,
}

impl Config {
    /// Load configuration from environment variables.
    /// DATABASE_URL defaults to "sqlite://fylge.db"
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://fylge.db".to_string());

        let listen_addr = std::env::var("LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
            .parse()
            .map_err(|_| ConfigError::Invalid("LISTEN_ADDR", "must be a valid socket address"))?;

        Ok(Config {
            listen_addr,
            database_url,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Invalid(&'static str, &'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Invalid(var, msg) => write!(f, "Invalid value for {}: {}", var, msg),
        }
    }
}

impl std::error::Error for ConfigError {}

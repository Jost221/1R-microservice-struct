use dotenv::dotenv;
use std::env;
use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub address: String,
    pub jwt_secret: String,
    pub jwt_exp_seconds: i64,
    pub refresh_exp_seconds: i64,
    pub redis_url: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("missing env var {0}")]
    MissingVar(String),
    #[error("parse int error: {0}")]
    ParseInt(#[from] ParseIntError),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8080);

        let address = env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::MissingVar("JWT_SECRET".into()))?;
        
        let jwt_exp_seconds = env::var("ACCESS_TTL_SECONDS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(15 * 60);

        let refresh_exp_seconds = env::var("REFRESH_TTL_SECONDS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(7 * 24 * 60 * 60);

        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

        Ok(Self {
            port,
            address,
            jwt_secret,
            jwt_exp_seconds,
            refresh_exp_seconds,
            redis_url,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

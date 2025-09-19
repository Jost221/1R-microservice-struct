use dotenv::dotenv;
use std::env;
use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub struct Config {
    // Существующие поля
    pub port: u16,
    pub address: String,
    pub jwt_secret: String,
    pub jwt_exp_seconds: i64,
    pub refresh_exp_seconds: i64,
    // Новые поля для Service Discovery
    pub service_registry_url: String, // URL Service Registry
    pub service_name: String, // Имя нашего сервиса
    pub health_check_interval: u64, // Интервал health check (сек)
    pub sql_controller_refresh: u64, // Интервал обновления SQL controller URL
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

        let address = env::var("ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingVar("JWT_SECRET".into()))?;

        let jwt_exp_seconds = env::var("ACCESS_TTL_SECONDS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(15 * 60);

        let refresh_exp_seconds = env::var("REFRESH_TTL_SECONDS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(7 * 24 * 60 * 60);

        // Новые параметры
        let service_registry_url = env::var("SERVICE_REGISTRY_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8500".to_string());
        let service_name = env::var("SERVICE_NAME")
            .unwrap_or_else(|_| "auth-service".to_string());
        let health_check_interval = env::var("HEALTH_CHECK_INTERVAL")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30); // 30 секунд
        let sql_controller_refresh = env::var("SQL_CONTROLLER_REFRESH")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60); // 60 секунд

        Ok(Self {
            port,
            address,
            jwt_secret,
            jwt_exp_seconds,
            refresh_exp_seconds,
            service_registry_url,
            service_name,
            health_check_interval,
            sql_controller_refresh,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

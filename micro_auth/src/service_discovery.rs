use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, interval, Instant};
use crate::config::Config;

#[derive(Serialize, Clone)]
pub struct HealthStatus {
    pub service_name: String,
    pub status: String,
    pub timestamp: String,
    pub host: String,
    pub port: u16,
    pub version: String,
}

#[derive(Deserialize)]
pub struct SqlControllerInfo { pub host: String, pub port: u16, pub status: String }

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("HTTP request failed: {0}")] HttpError(#[from] reqwest::Error),
    #[error("SQL Controller not found")] SqlControllerNotFound,
    #[error("Service registry unavailable")] RegistryUnavailable,
    #[error("Invalid response format")] InvalidResponse,
}

pub struct ServiceDiscoveryClient {
    client: Client,
    config: Arc<Config>,
    sql_controller_url: Arc<RwLock<Option<String>>>,
    last_health_check: Arc<RwLock<Option<Instant>>>,
}

impl ServiceDiscoveryClient {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            client: Client::builder().timeout(Duration::from_secs(10)).build().unwrap(),
            config,
            sql_controller_url: Arc::new(RwLock::new(None)),
            last_health_check: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_sql_controller_url(&self) -> Result<String, ServiceError> {
        if let Some(url) = self.sql_controller_url.read().unwrap().clone() {
            return Ok(url);
        }
        self.discover_sql_controller().await
    }

    async fn discover_sql_controller(&self) -> Result<String, ServiceError> {
        let url = format!("{}/api/service/sql-controller", self.config.service_registry_url);
        let res = self.client.get(&url).send().await?;
        if res.status().is_success() {
            let info = res.json::<SqlControllerInfo>().await?;
            let full = format!("http://{}:{}", info.host, info.port);
            *self.sql_controller_url.write().unwrap() = Some(full.clone());
            Ok(full)
        } else {
            Err(ServiceError::SqlControllerNotFound)
        }
    }

    pub async fn send_health_status(&self) -> Result<(), ServiceError> {
        let status = HealthStatus {
            service_name: self.config.service_name.clone(),
            status: "healthy".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            host: self.config.address.clone(),
            port: self.config.port,
            version: "1.0.0".into(),
        };
        let url = format!("{}/api/health", self.config.service_registry_url);
        let res = self.client.post(&url).json(&status).send().await?;
        if res.status().is_success() {
            *self.last_health_check.write().unwrap() = Some(Instant::now());
            Ok(())
        } else {
            Err(ServiceError::RegistryUnavailable)
        }
    }

    pub async fn start_periodic_tasks(&self) {
        let mut hc = interval(Duration::from_secs(self.config.health_check_interval));
        let mut dc = interval(Duration::from_secs(self.config.sql_controller_refresh));
        loop {
            tokio::select! {
                _ = hc.tick() => { let _ = self.send_health_status().await; }
                _ = dc.tick() => { let _ = self.discover_sql_controller().await; }
            }
        }
    }
}
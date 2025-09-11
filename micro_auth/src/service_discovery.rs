use reqwest::Client;
use serde::{Deserialize, Serialize};
// Убираем эту строку: use serde_json::json;
use std::sync::Arc;
use tokio::time::{Duration, interval, Instant};
use std::sync::RwLock;
use crate::config::Config;

// Структуры для взаимодействия с Service Registry
#[derive(Serialize, Clone)]
pub struct HealthStatus {
    pub service_name: String,
    pub status: String,
    pub timestamp: String,
    pub host: String,
    pub port: u16,
    pub version: String,
}

#[derive(Deserialize, Clone)]
pub struct SqlControllerInfo {
    pub host: String,
    pub port: u16,
    pub status: String,
}

pub struct ServiceDiscoveryClient {
    client: Client,
    config: Arc<Config>,
    sql_controller_url: Arc<RwLock<Option<String>>>,
    last_health_check: Arc<RwLock<Option<Instant>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("SQL Controller not found")]
    SqlControllerNotFound,
    #[error("Service registry unavailable")]
    RegistryUnavailable,
    #[error("Invalid response format")]
    InvalidResponse,
}

impl ServiceDiscoveryClient {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            config,
            sql_controller_url: Arc::new(RwLock::new(None)),
            last_health_check: Arc::new(RwLock::new(None)),
        }
    }
    
    // Получение адреса SQL Controller
    pub async fn get_sql_controller_url(&self) -> Result<String, ServiceError> {
        // Проверяем кэш
        if let Some(url) = self.sql_controller_url.read().unwrap().clone() {
            return Ok(url);
        }
        
        // Если в кэше нет, запрашиваем у Service Registry
        self.discover_sql_controller().await
    }
    
    // Запрос SQL Controller у Service Registry
    async fn discover_sql_controller(&self) -> Result<String, ServiceError> {
        println!("Discovering SQL Controller from Service Registry...");
        
        let url = format!("{}/api/service/sql-controller", self.config.service_registry_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<SqlControllerInfo>().await {
                        Ok(controller_info) => {
                            let controller_url = format!("http://{}:{}", 
                                controller_info.host, 
                                controller_info.port);
                            
                            // Обновляем кэш
                            *self.sql_controller_url.write().unwrap() = Some(controller_url.clone());
                            
                            println!("✅ Found SQL Controller at: {}", controller_url);
                            Ok(controller_url)
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to parse SQL Controller response: {}", e);
                            Err(ServiceError::InvalidResponse)
                        }
                    }
                } else {
                    eprintln!("❌ Service Registry returned status: {}", response.status());
                    Err(ServiceError::SqlControllerNotFound)
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to Service Registry: {}", e);
                Err(ServiceError::RegistryUnavailable)
            }
        }
    }
    
    // Отправка health check в Service Registry
    pub async fn send_health_status(&self) -> Result<(), ServiceError> {
        let health_status = HealthStatus {
            service_name: self.config.service_name.clone(),
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            host: self.config.address.clone(),
            port: self.config.port,
            version: "1.0.0".to_string(),
        };
        
        let url = format!("{}/api/health", self.config.service_registry_url);
        
        match self.client
            .post(&url)
            .json(&health_status)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    *self.last_health_check.write().unwrap() = Some(Instant::now());
                    println!("✅ Health status sent successfully");
                    Ok(())
                } else {
                    eprintln!("⚠️ Health check failed with status: {}", response.status());
                    Err(ServiceError::RegistryUnavailable)
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to send health status: {}", e);
                Err(ServiceError::HttpError(e))
            }
        }
    }
    
    // Периодические задачи
    pub async fn start_periodic_tasks(&self) {
        let health_interval = self.config.health_check_interval;
        let discovery_interval = self.config.sql_controller_refresh;
        
        let mut health_timer = interval(Duration::from_secs(health_interval));
        let mut discovery_timer = interval(Duration::from_secs(discovery_interval));
        
        loop {
            tokio::select! {
                _ = health_timer.tick() => {
                    if let Err(e) = self.send_health_status().await {
                        eprintln!("Health check failed: {}", e);
                    }
                }
                _ = discovery_timer.tick() => {
                    if let Err(e) = self.discover_sql_controller().await {
                        eprintln!("SQL Controller discovery failed: {}", e);
                    }
                }
            }
        }
    }
    
    // Получение статистики для мониторинга
    pub fn get_status(&self) -> ServiceStatus {
        ServiceStatus {
            sql_controller_cached: self.sql_controller_url.read().unwrap().is_some(),
            last_health_check: self.last_health_check.read().unwrap().clone(),
            service_name: self.config.service_name.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ServiceStatus {
    pub sql_controller_cached: bool,
    pub last_health_check: Option<Instant>,
    pub service_name: String,
}

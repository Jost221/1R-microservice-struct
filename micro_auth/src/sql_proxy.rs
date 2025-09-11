use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::time::Duration;
use crate::service_discovery::{ServiceDiscoveryClient, ServiceError};

#[derive(Serialize, Debug)]
pub struct RedisCommand {
    pub operation: String,  // "get", "set", "set_nx", "del", "set_ex"
    pub key: String,
    pub value: Option<String>,
    pub ttl: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct RedisResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

pub struct SqlControllerProxy {
    client: Client,
    service_discovery: Arc<ServiceDiscoveryClient>,
}

impl SqlControllerProxy {
    pub fn new(service_discovery: Arc<ServiceDiscoveryClient>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Failed to create HTTP client"),
            service_discovery,
        }
    }
    
    // Выполнение команды Redis через SQL Controller
    pub async fn execute_redis_command(&self, command: RedisCommand) -> Result<Value, ServiceError> {
        let controller_url = self.service_discovery.get_sql_controller_url().await?;
        
        // Попытки повтора при ошибках
        let mut attempts = 3;
        
        while attempts > 0 {
            match self.send_command(&controller_url, &command).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts -= 1;
                    if attempts == 0 {
                        return Err(e);
                    }
                    println!("⚠️ Command failed, retrying... (attempts left: {})", attempts);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
        
        Err(ServiceError::SqlControllerNotFound)
    }
    
    async fn send_command(&self, base_url: &str, command: &RedisCommand) -> Result<Value, ServiceError> {
        let url = format!("{}/api/redis/command", base_url);
        
        println!("📡 Sending Redis command: {:?}", command);
        
        let response = self.client
            .post(&url)
            .json(command)
            .send()
            .await?;
        
        if response.status().is_success() {
            let redis_response: RedisResponse = response.json().await?;
            
            if redis_response.success {
                Ok(redis_response.data.unwrap_or(Value::Null))
            } else {
                eprintln!("❌ Redis command failed: {:?}", redis_response.error);
                Err(ServiceError::InvalidResponse)
            }
        } else {
            eprintln!("❌ HTTP error from SQL Controller: {}", response.status());
            Err(ServiceError::SqlControllerNotFound)
        }
    }
    
    // Удобные методы для основных операций
    pub async fn redis_get(&self, key: &str) -> Result<Option<String>, ServiceError> {
        let command = RedisCommand {
            operation: "get".to_string(),
            key: key.to_string(),
            value: None,
            ttl: None,
        };
        
        let result = self.execute_redis_command(command).await?;
        
        match result {
            Value::String(s) => Ok(Some(s)),
            Value::Null => Ok(None),
            _ => Err(ServiceError::InvalidResponse),
        }
    }
    
    pub async fn redis_set_nx(&self, key: &str, value: &str) -> Result<bool, ServiceError> {
        let command = RedisCommand {
            operation: "set_nx".to_string(),
            key: key.to_string(),
            value: Some(value.to_string()),
            ttl: None,
        };
        
        let result = self.execute_redis_command(command).await?;
        
        match result {
            Value::Bool(b) => Ok(b),
            _ => Err(ServiceError::InvalidResponse),
        }
    }
    
    pub async fn redis_set_ex(&self, key: &str, value: &str, ttl: u64) -> Result<(), ServiceError> {
        let command = RedisCommand {
            operation: "set_ex".to_string(),
            key: key.to_string(),
            value: Some(value.to_string()),
            ttl: Some(ttl),
        };
        
        self.execute_redis_command(command).await?;
        Ok(())
    }
    
    pub async fn redis_del(&self, key: &str) -> Result<(), ServiceError> {
        let command = RedisCommand {
            operation: "del".to_string(),
            key: key.to_string(),
            value: None,
            ttl: None,
        };
        
        self.execute_redis_command(command).await?;
        Ok(())
    }
}

use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::time::Duration;
use crate::service_discovery::{ServiceDiscoveryClient, ServiceError};

#[derive(Serialize)]
pub struct RedisCommand {
    pub operation: String,
    pub key: String,
    pub value: Option<String>,
    pub ttl: Option<u64>,
}

#[derive(Deserialize)]
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
    pub fn new(sd: Arc<ServiceDiscoveryClient>) -> Self {
        Self { client: Client::builder().timeout(Duration::from_secs(5)).build().unwrap(), service_discovery: sd }
    }

    pub async fn execute_redis_command(&self, cmd: RedisCommand) -> Result<Value, ServiceError> {
        let url = self.service_discovery.get_sql_controller_url().await?;
        let mut attempts = 3;
        while attempts > 0 {
            let res = self.client.post(&format!("{}/api/redis/command", url)).json(&cmd).send().await;
            match res {
                Ok(r) if r.status().is_success() => {
                    let rr = r.json::<RedisResponse>().await?;
                    if rr.success { return Ok(rr.data.unwrap_or(Value::Null)); }
                    else { return Err(ServiceError::InvalidResponse); }
                }
                _ => { attempts -= 1; tokio::time::sleep(Duration::from_millis(100)).await; }
            }
        }
        Err(ServiceError::SqlControllerNotFound)
    }

    pub async fn redis_get(&self, key: &str) -> Result<Option<String>, ServiceError> {
        let v = self.execute_redis_command(RedisCommand { operation: "get".into(), key: key.into(), value: None, ttl: None }).await?;
        match v { Value::String(s) => Ok(Some(s)), Value::Null => Ok(None), _ => Err(ServiceError::InvalidResponse) }
    }

    pub async fn redis_set_nx(&self, key: &str, value: &str) -> Result<bool, ServiceError> {
        let v = self.execute_redis_command(RedisCommand { operation: "set_nx".into(), key: key.into(), value: Some(value.into()), ttl: None }).await?;
        match v { Value::Bool(b) => Ok(b), _ => Err(ServiceError::InvalidResponse) }
    }

    pub async fn redis_set_ex(&self, key: &str, value: &str, ttl: u64) -> Result<(), ServiceError> {
        self.execute_redis_command(RedisCommand { operation: "set_ex".into(), key: key.into(), value: Some(value.into()), ttl: Some(ttl) }).await?;
        Ok(())
    }

    pub async fn redis_set(&self, key: &str, value: &str) -> Result<(), ServiceError> {
        self.execute_redis_command(RedisCommand { operation: "set".into(), key: key.into(), value: Some(value.into()), ttl: None }).await?;
        Ok(())
    }

    pub async fn redis_del(&self, key: &str) -> Result<(), ServiceError> {
        self.execute_redis_command(RedisCommand { operation: "del".into(), key: key.into(), value: None, ttl: None }).await?;
        Ok(())
    }
}
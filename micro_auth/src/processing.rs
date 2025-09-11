use argon2::{Argon2, password_hash::{SaltString, PasswordHasher, PasswordHash, PasswordVerifier}};
use std::sync::Arc;
use thiserror::Error;
use rand_core::OsRng;
use crate::sql_proxy::SqlControllerProxy;
use crate::service_discovery::ServiceDiscoveryClient;

#[derive(Clone)]
pub struct AppState {
    pub sql_proxy: Arc<SqlControllerProxy>,
    pub service_discovery: Arc<ServiceDiscoveryClient>,
    pub jwt_secret: String,
    pub jwt_exp_seconds: i64,
    pub refresh_exp_seconds: i64,
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user already exists")]
    Exists,
    #[error("user not found")]
    NotFound,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("service error: {0}")]
    Service(#[from] crate::service_discovery::ServiceError),
    #[error("hash error")]
    Hash,
}

impl AppState {
    pub async fn create_user(&self, username: &str, password: &str) -> Result<(), UserError> {
        // Хешируем пароль
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| UserError::Hash)?
            .to_string();
        
        let key = format!("user:{}", username);
        
        // Используем SQL Proxy вместо прямого обращения к Redis
        let created = self.sql_proxy.redis_set_nx(&key, &password_hash).await?;
        
        if !created {
            return Err(UserError::Exists);
        }
        
        println!("✅ User '{}' created successfully", username);
        Ok(())
    }
    
    pub async fn verify_user(&self, username: &str, password: &str) -> Result<(), UserError> {
        let key = format!("user:{}", username);
        
        // Получаем хеш пароля через SQL Proxy
        let stored_hash = self.sql_proxy.redis_get(&key).await?;
        
        let Some(phc) = stored_hash else { 
            return Err(UserError::NotFound) 
        };
        
        let parsed = PasswordHash::new(&phc).map_err(|_| UserError::Hash)?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| UserError::InvalidCredentials)
            .map(|_| ())
    }
    
    pub async fn store_refresh_jti(&self, jti: &str, username: &str, ttl_seconds: i64) -> Result<(), UserError> {
        let key = format!("refresh:{}", jti);
        self.sql_proxy.redis_set_ex(&key, username, ttl_seconds as u64).await?;
        Ok(())
    }
    
    pub async fn check_refresh_jti(&self, jti: &str) -> Result<Option<String>, UserError> {
        let key = format!("refresh:{}", jti);
        let result = self.sql_proxy.redis_get(&key).await?;
        Ok(result)
    }
    
    pub async fn revoke_refresh_jti(&self, jti: &str) -> Result<(), UserError> {
        let key = format!("refresh:{}", jti);
        self.sql_proxy.redis_del(&key).await?;
        Ok(())
    }
}

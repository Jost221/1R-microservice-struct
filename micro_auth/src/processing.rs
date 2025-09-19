use argon2::{Argon2, password_hash::{SaltString, PasswordHasher, PasswordHash, PasswordVerifier}};
use std::sync::Arc;
use thiserror::Error;
use rand_core::OsRng;
use crate::sql_proxy::SqlControllerProxy;
use crate::service_discovery::ServiceDiscoveryClient;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user already exists")] Exists,
    #[error("user not found")] NotFound,
    #[error("invalid credentials")] InvalidCredentials,
    #[error("service error: {0}")] Service(#[from] crate::service_discovery::ServiceError),
    #[error("hash error")] Hash,
    #[error("invalid data format")] InvalidData,
    #[error("access denied - insufficient permissions")] AccessDenied,
}

#[derive(Clone)]
pub struct AppState {
    pub sql_proxy: Arc<SqlControllerProxy>,
    pub service_discovery: Arc<ServiceDiscoveryClient>,
    pub jwt_secret: String,
    pub jwt_exp_seconds: i64,
    pub refresh_exp_seconds: i64,
}

impl AppState {
    pub async fn create_user(&self, username: &str, password: &str) -> Result<(), UserError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| UserError::Hash)?
            .to_string();
        let key = format!("user:{}", username);
        if !self.sql_proxy.redis_set_nx(&key, &password_hash).await? {
            return Err(UserError::Exists);
        }
        let _ = self.assign_user_roles(username, vec!["user".into()]).await;
        Ok(())
    }

    pub async fn verify_user(&self, username: &str, password: &str) -> Result<(), UserError> {
        let key = format!("user:{}", username);
        let stored = self.sql_proxy.redis_get(&key).await?;
        let phc = stored.ok_or(UserError::NotFound)?;
        let parsed = PasswordHash::new(&phc).map_err(|_| UserError::Hash)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed)
            .map_err(|_| UserError::InvalidCredentials)?;
        Ok(())
    }

    // Refresh JTI storage, revocation, etc.
    pub async fn store_refresh_jti(&self, jti: &str, user: &str, ttl: i64) -> Result<(), UserError> {
        self.sql_proxy.redis_set_ex(&format!("refresh:{}", jti), user, ttl as u64).await?;
        Ok(())
    }
    pub async fn check_refresh_jti(&self, jti: &str) -> Result<Option<String>, UserError> {
        Ok(self.sql_proxy.redis_get(&format!("refresh:{}", jti)).await?)
    }
    pub async fn revoke_refresh_jti(&self, jti: &str) -> Result<(), UserError> {
        self.sql_proxy.redis_del(&format!("refresh:{}", jti)).await?;
        Ok(())
    }

    // Role management
    pub async fn get_user_roles(&self, username: &str) -> Result<Vec<String>, UserError> {
        let key = format!("user_roles:{}", username);
        if let Some(json) = self.sql_proxy.redis_get(&key).await? {
            serde_json::from_str(&json).map_err(|_| UserError::InvalidData)
        } else {
            Ok(vec!["user".into()])
        }
    }

    pub async fn assign_user_roles(&self, username: &str, roles: Vec<String>) -> Result<(), UserError> {
        let key = format!("user:{}", username);
        if self.sql_proxy.redis_get(&key).await?.is_none() {
            return Err(UserError::NotFound);
        }
        let key = format!("user_roles:{}", username);
        let json = serde_json::to_string(&roles).map_err(|_| UserError::InvalidData)?;
        self.sql_proxy.redis_set(&key, &json).await?;
        Ok(())
    }

    pub async fn user_has_role(&self, username: &str, role: &str) -> Result<bool, UserError> {
        Ok(self.get_user_roles(username).await?.contains(&role.to_string()))
    }
}
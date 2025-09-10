
use argon2::{Argon2, password_hash::{SaltString, PasswordHasher, PasswordHash, PasswordVerifier}};
use redis::AsyncCommands;
use std::sync::Arc;
use redis::aio::ConnectionManager;
use thiserror::Error;
use rand_core::OsRng;

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<ConnectionManager>,
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
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("hash error")]
    Hash,
}

impl AppState {
    pub async fn create_user(&self, username: &str, password: &str) -> Result<(), UserError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| UserError::Hash)?
            .to_string();

        let key = format!("user:{username}");
        let mut conn = (*self.redis).clone();
        // SETNX to avoid overwriting
        let created: bool = conn.set_nx(key, password_hash).await?;
        if !created {
            return Err(UserError::Exists);
        }
        Ok(())
    }

    pub async fn verify_user(&self, username: &str, password: &str) -> Result<(), UserError> {
        let key = format!("user:{username}");
        let mut conn = (*self.redis).clone();
        let stored: Option<String> = conn.get(key).await?;
        let Some(phc) = stored else { return Err(UserError::NotFound) };
        let parsed = PasswordHash::new(&phc).map_err(|_| UserError::Hash)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| UserError::InvalidCredentials)
            .map(|_| ())
    }

    /// Store refresh token jti -> username with TTL
    pub async fn store_refresh_jti(&self, jti: &str, username: &str, ttl_seconds: i64) -> Result<(), redis::RedisError> {
        let key = format!("refresh:{jti}");
        let mut conn = (*self.redis).clone();
        let _: () = conn.set_ex(key, username, ttl_seconds as u64).await?;
        Ok(())
    }

    pub async fn check_refresh_jti(&self, jti: &str) -> Result<Option<String>, redis::RedisError> {
        let key = format!("refresh:{jti}");
        let mut conn = (*self.redis).clone();
        let v: Option<String> = conn.get(key).await?;
        Ok(v)
    }

    pub async fn revoke_refresh_jti(&self, jti: &str) -> Result<(), redis::RedisError> {
        let key = format!("refresh:{jti}");
        let mut conn = (*self.redis).clone();
        let _: () = conn.del(key).await?;
        Ok(())
    }
}

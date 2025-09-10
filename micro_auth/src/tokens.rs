
use chrono::{Utc, Duration};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode, TokenData};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,   // username
    pub iat: i64,
    pub exp: i64,
    pub jti: String,   // token id
    pub typ: String,   // access or refresh
}

/// Create a JWT with given ttl seconds and typ ("access" or "refresh").
pub fn create_token(
    secret: &str,
    username: &str,
    ttl_seconds: i64,
    typ: &str,
) -> Result<(String, String, i64), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_seconds);
    let jti = Uuid::new_v4().to_string();

    let claims = Claims {
        sub: username.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        jti: jti.clone(),
        typ: typ.to_string(),
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))?;
    Ok((token, jti, exp.timestamp()))
}

pub fn validate_token(secret: &str, token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &validation)
}

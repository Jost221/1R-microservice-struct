use chrono::{Utc, Duration};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode, TokenData};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
    pub typ: String,
}

pub fn create_token(secret: &str, user: &str, ttl: i64, typ: &str) -> Result<(String,String,i64),jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(ttl)).timestamp();
    let jti = Uuid::new_v4().to_string();
    let claims = Claims { sub: user.into(), iat: now.timestamp(), exp, jti: jti.clone(), typ: typ.into() };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))?;
    Ok((token, jti, exp))
}

pub fn validate_token(secret: &str, token: &str) -> Result<TokenData<Claims>,jsonwebtoken::errors::Error> {
    let mut v = Validation::default();
    v.validate_exp = true;
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &v)
}
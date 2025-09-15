use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use redis_derive::{ToRedisArgs, FromRedisValue};


#[derive(Serialize, Deserialize, Debug, FromRow, ToRedisArgs)]
pub struct User {
    pub name: Option<String>,
    pub id: i32,
}
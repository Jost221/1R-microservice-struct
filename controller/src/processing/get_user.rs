use sqlx::{self, Pool, Postgres};
use redis::{self, AsyncCommands, Client, Commands};

use crate::sql_struct::*;

static TIME_S:i32 = 6000; 

pub async fn get_users(pool : &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error>{
    let users = sqlx::query_file_as!(User, "src/requests/get_users.sql")
    .fetch_all(pool)
    .await?;

    Ok(users)
}

pub async fn get_user_by_id(pool: &Pool<Postgres>, redis_client: Client, id: i32) -> Result<User, sqlx::Error>{
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await.unwrap() ;

    let serch_key = format!("user_{}", id);
    let data: Result<User, redis::RedisError> = redis_conn.get(serch_key).await;
    match data {
        Ok(res) => {
            let _ = redis_conn.expire(serch_key, TIME_S);
            return Ok(res);
        }
        Err(_) =>{

            let user = sqlx::query_file_as!(User, "src/requests/get_user_by_id.sql")
                            .fetch_one(pool)
                            .await?;
            
            if let Err(e) = redis_client.set_ex(serch_key, user, TIME_S) {
                dbg!(e);
            }
            
            return Ok(user);
        }        
    }
}
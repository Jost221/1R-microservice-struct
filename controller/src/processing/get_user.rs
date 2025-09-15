use redis::{self, AsyncCommands, Client, Commands};
use sqlx::{self, Pool, Postgres};

use crate::sql_struct::*;

//time life value in redis
static TIME_S: u64 = 6000;

pub async fn get_users(pool: &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_file_as!(User, "src/requests/get_users.sql")
        .fetch_all(pool)
        .await?;

    Ok(users)
}

pub async fn get_user_by_id(
    pool: &Pool<Postgres>,
    redis_client: &Client,
    id: u64,
) -> Result<User, sqlx::Error> {
    let mut redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let serch_key = format!("user_{}", id);
    let data: Result<String, redis::RedisError> = redis_conn.get(serch_key.clone()).await;
    match data {
        Ok(res) => {
            let res_update: Result<bool, redis::RedisError> = redis_conn.expire(serch_key, TIME_S as i64).await;
            match res_update {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Ошибка Redis: {:?}", err);
                }
            };

            Ok(User {
                name: Some(res),
                id: id as i32,
            })
        }
        Err(_) => {
            let user = sqlx::query_file_as!(User, "src/requests/get_user_by_id.sql")
                .fetch_one(pool)
                .await?;

            let res_write: Result<bool, redis::RedisError> = redis_conn.set_ex(serch_key, user.name.clone(), TIME_S).await;
            match res_write {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Ошибка Redis: {:?}", err);
                }
            };

            Ok(user)
        }
    }
}

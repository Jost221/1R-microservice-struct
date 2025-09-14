use sqlx::{self, Pool, Postgres};
use redis::{self, Client, Commands};

use crate::sql_struct::*;

pub async fn get_users(pool : &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error>{
    let users = sqlx::query_file_as!(User, "src/requests/get_users.sql")
    .fetch_all(pool)
    .await?;

    Ok(users)
}

// pub async fn get_user_by_id(pool: &Pool<Postgres>, redis_client: Client, id: i32) -> Result<User, sqlx::Error>{
//     let redis_conn = redis_client.get_connection().unwrap();

//     if let Ok(data) = redis_conn.get(format!("user_{}", id)) {

//     }

//     Ok(User{
//         id: 4,
//         name: Some("biba".to_string())
//     })
// }
use axum::{extract::State, routing::*, Router};
use tokio::net::TcpListener;
use sqlx::PgPool;
use std::sync::Arc;

mod handlers;
mod config;
mod sql_struct;
mod api_static;
mod processing;

use crate::handlers::*;
use crate::api_static::*;
#[tokio::main]
async fn main() {

    let config = config::Config::from_env().expect("Failed to load config");

    let pool = PgPool::connect(config.sql_url.as_str()).await.unwrap();
    let appstate = Arc::new(AppState{
        sql_pool:pool
    });

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/getUsers", get(get_users))
        .with_state(appstate.clone());


    let listener = TcpListener::bind(config.address_to_string())
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    // //подтягиваем .env файл
    // dotenv().expect("Failed to load .env file");
    // // Подключение к БД (пример для PostgreSQL)
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // println!("start connect");
    // let pool = PgPool::connect(database_url.as_str()).await?;
    // println!("send request");
    // // Запрос на получение всех пользователей
    // let users  = sqlx::query_file_as!(Users, "src/requests/get_user.sql")
    //     .fetch_all(&pool)
    //     .await?;

    // dbg!(users);

    // Ok(())
}

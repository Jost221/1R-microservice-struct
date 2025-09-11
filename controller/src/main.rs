use axum::{routing::*, Router};
use tokio::net::TcpListener;

mod handlers;
mod config;
mod sql_struct;
mod api_response;
mod api_json_struct;

use crate::handlers::*;

#[tokio::main]
async fn main() {

    let config = config::Config::from_env().expect("Failed to load config");

    let app = Router::new()
        .route("/", get(root_handler));

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

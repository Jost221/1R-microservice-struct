use axum::{routing::*, Router};
use tokio::net::TcpListener;

mod handlers;
mod config;
mod processing;
mod json_struct;
mod api_response;

use handlers::*;

#[tokio::main]
async fn main() {
    // Загрузка конфигурации
    let config = config::Config::from_env().expect("Failed to load config");

    // Маршруты
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/services", post(set_id))
        .route("/services", get(get_microservices))
        .route("/services", delete(remove_data));
    // Запуск сервера
    let listener = TcpListener::bind(config.to_string())
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
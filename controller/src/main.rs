use axum::{routing::*, Router};
use tokio::net::TcpListener;
use sqlx::PgPool;
use std::sync::Arc;
use redis;

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

    //init db
    let pool = PgPool::connect(config.sql_url.as_str()).await.unwrap();
    
    //init redis
    let redis_client = redis::Client::open(config.redis_url.as_str()).unwrap();


    let appstate = Arc::new(AppState{
        sql_pool:pool,
        redis_pool: redis_client
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
}

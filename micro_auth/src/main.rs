use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use tokio::net::TcpListener;
use std::sync::Arc;
use redis::aio::ConnectionManager;
use redis::Client as RedisClient;

mod handlers;
mod config;
mod processing;
mod json_struct;
mod tokens;
mod auth;

use config::Config;
use processing::AppState;
use handlers::*;

#[tokio::main]
async fn main() {
    // config
    let cfg = Config::from_env().expect("config");

    // redis  
    let client = RedisClient::open(cfg.redis_url.clone()).expect("redis url");
    let manager = ConnectionManager::new(client).await.expect("redis connect");

    let app_state = Arc::new(AppState {
        redis: Arc::new(manager),
        jwt_secret: cfg.jwt_secret.clone(),
        jwt_exp_seconds: cfg.jwt_exp_seconds,
        refresh_exp_seconds: cfg.refresh_exp_seconds,
    });

    // public routes
    let public = Router::new()
        .route("/", get(root_handler))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .with_state(app_state.clone());

    // protected routes
    let protected = Router::new()
        .route("/services", get(get_microservices))
        .layer(middleware::from_fn_with_state(app_state.clone(), auth::my_middleware));

    let app = Router::new()
        .merge(public)
        .merge(protected);

    let bind_addr = cfg.bind_addr();
    let listener = TcpListener::bind(bind_addr.clone()).await.expect("Failed to bind");
    println!("listening on {}", bind_addr);
    axum::serve(listener, app).await.unwrap();
}

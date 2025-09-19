use axum::{
    routing::{get, post},
    Router, middleware, Json,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use serde_json::json;

mod handlers;
mod config;
mod processing;
mod json_struct;
mod tokens;
mod auth;
mod service_discovery;
mod sql_proxy;

use config::Config;
use processing::AppState;
use handlers::*;
use service_discovery::ServiceDiscoveryClient;
use sql_proxy::SqlControllerProxy;

#[tokio::main]
async fn main() {
    println!("🚀 Starting Auth Service...");

    // Инициализация
    let cfg = Arc::new(Config::from_env().expect("Failed to load config"));
    let sd = Arc::new(ServiceDiscoveryClient::new(cfg.clone()));
    let proxy = Arc::new(SqlControllerProxy::new(sd.clone()));
    let state = Arc::new(AppState {
        sql_proxy: proxy,
        service_discovery: sd.clone(),
        jwt_secret: cfg.jwt_secret.clone(),
        jwt_exp_seconds: cfg.jwt_exp_seconds,
        refresh_exp_seconds: cfg.refresh_exp_seconds,
    });

    // Фоновые задачи
    let service_discovery_bg = sd.clone();
    tokio::spawn(async move {
        service_discovery_bg.start_periodic_tasks().await;
    });

    // Маршруты
    let public = Router::new()
        .route("/", get(root_handler))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/health", get(|| async { Json(json!({"status":"healthy"})) }))
        .with_state(state.clone());

    let protected = Router::new()
        .route("/services", get(get_microservices))
        .route("/user/roles", get(get_current_user_roles))
        .route("/admin/assign-roles", post(assign_roles))
        .layer(middleware::from_fn_with_state(state.clone(), auth::my_middleware))
        .with_state(state.clone());

    let app = public.merge(protected);

    // Запуск сервера - используем нативный TCP listener
    let bind_addr = cfg.bind_addr();
    let listener = TcpListener::bind(&bind_addr).await
        .expect("Failed to bind to address");

    println!("🎯 Auth Service listening on {}", bind_addr);
    println!("📋 Available endpoints:");
    println!("   GET / - Welcome message");
    println!("   POST /register - User registration");
    println!("   POST /login - User authentication");
    println!("   POST /refresh - Token refresh");
    println!("   GET /health - Health check");
    println!("   GET /services - Protected endpoint");
    println!("   GET /user/roles - Get current user roles");
    println!("   POST /admin/assign-roles - Assign user roles (admin only)");

    // ЕДИНСТВЕННЫЙ рабочий способ для Axum 0.7
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

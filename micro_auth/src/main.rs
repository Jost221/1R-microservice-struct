use axum::{
    routing::{get, post},
    Router,
    middleware,
    Json,
    extract::State,
};
use tokio::net::TcpListener;
use std::sync::Arc;
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
    
    // Загружаем конфигурацию
    let cfg = Arc::new(Config::from_env().expect("Failed to load config"));
    println!("✅ Configuration loaded");
    
    // Создаем Service Discovery Client
    let service_discovery = Arc::new(ServiceDiscoveryClient::new(cfg.clone()));
    println!("✅ Service Discovery Client created");
    
    // Создаем SQL Controller Proxy
    let sql_proxy = Arc::new(SqlControllerProxy::new(service_discovery.clone()));
    println!("✅ SQL Controller Proxy created");
    
    // Создаем состояние приложения
    let app_state = Arc::new(AppState {
        sql_proxy,
        service_discovery: service_discovery.clone(),
        jwt_secret: cfg.jwt_secret.clone(),
        jwt_exp_seconds: cfg.jwt_exp_seconds,
        refresh_exp_seconds: cfg.refresh_exp_seconds,
    });
    
    // Пытаемся найти SQL Controller при запуске
    println!("🔍 Looking for SQL Controller...");
    match service_discovery.get_sql_controller_url().await {
        Ok(url) => println!("✅ SQL Controller found at: {}", url),
        Err(e) => println!("⚠️ SQL Controller not found yet: {}", e),
    }
    
    // Отправляем первый health check
    println!("📡 Sending initial health status...");
    if let Err(e) = service_discovery.send_health_status().await {
        println!("⚠️ Initial health check failed: {}", e);
    }
    
    // Запускаем фоновые задачи
    let service_discovery_bg = service_discovery.clone();
    tokio::spawn(async move {
        println!("🔄 Starting periodic tasks...");
        service_discovery_bg.start_periodic_tasks().await;
    });
    
    // Настраиваем маршруты
    let public = Router::new()
        .route("/", get(root_handler))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .with_state(app_state.clone());
    
    let protected = Router::new()
        .route("/services", get(get_microservices))
        .layer(middleware::from_fn_with_state(app_state.clone(), auth::my_middleware));
    
    let app = Router::new()
        .merge(public)
        .merge(protected);
    
    // Запускаем сервер
    let bind_addr = cfg.bind_addr();
    let listener = TcpListener::bind(bind_addr.clone()).await.expect("Failed to bind");
    
    println!("🎯 Auth Service listening on {}", bind_addr);
    println!("📋 Available endpoints:");
    println!("   GET  /           - Welcome message");
    println!("   POST /register   - User registration");
    println!("   POST /login      - User authentication");
    println!("   POST /refresh    - Token refresh");
    println!("   GET  /health     - Health check");
    println!("   GET  /status     - Service status");
    println!("   GET  /services   - Protected endpoint");
    
    axum::serve(listener, app).await.unwrap();
}

// Новые хендлеры для мониторинга
async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "auth-service",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    }))
}

async fn status_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let status = state.service_discovery.get_status();
    
    Json(json!({
        "service_name": status.service_name,
        "sql_controller_cached": status.sql_controller_cached,
        "last_health_check": status.last_health_check.map(|t| format!("{:?} ago", t.elapsed())),
        "uptime": "running",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

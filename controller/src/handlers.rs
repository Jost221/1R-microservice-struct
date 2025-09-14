use axum::{Json, extract::State};
use std::sync::Arc;
use serde_json::json;

use crate::api_static::*;
use crate::processing;

pub async fn root_handler() -> Json<Message> {
    Json(Message {
        message: "Welcome to my microservice!".to_string(),
    })
}

pub async fn get_users(State(app_state): State<Arc<AppState>>) -> () {
    processing::get_user::get_users();
}
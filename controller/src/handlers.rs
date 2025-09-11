use axum::Json;

use crate::api_response::*;
use crate::api_json_struct::*;

pub async fn root_handler() -> Json<Message> {
    Json(Message {
        message: "Welcome to my microservice!".to_string(),
    })
}


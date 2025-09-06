use axum::Json;

use crate::api_response::*;
use crate::json_struct::*;
use crate::processing::*;

pub async fn root_handler() -> Json<Message> {
    Json(Message {
        message: "Welcome to my microservice!".to_string(),
    })
}

pub async fn set_id(Json(payload): Json<WriteData>) -> Result<Json<Message>, AppError> { 
    match set_data(payload).await {
        Ok(msg) => {
            Ok(Json( Message {
                message: msg
            }))
        },
        Err(_) => Err(AppError::InternalError)
    }
}

pub async fn get_microservices() -> Result<Json<Data>, AppError> {
    match get_writed().await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(AppError::ErrorReadFile),
    }

}

pub async fn remove_data(Json(payload): Json<WriteData>) -> Result<Json<Message>, AppError> {
    match remove_data_from_file(payload).await {
        Ok(msg) => Ok(Json(Message{message:msg})),
        Err(_) => Err(AppError::ErrorReadFile)
    }
}
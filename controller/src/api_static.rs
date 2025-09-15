use axum::{
    Json,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use redis::Client;
use serde::Serialize;
use sqlx::{Pool, Postgres};

// Определяем структуру для ответа об ошибке
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub sql_pool: Pool<Postgres>,
    pub redis_pool: Client
}

// Определяем собственный тип ошибки
#[derive(Debug)]
pub enum AppError {
    InternalError,
    ErrorWrokWithDB
}

// Реализуем IntoResponse для преобразования ошибки в HTTP-ответ
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: "Internal server error".to_string(),
                    details: Some("Something went wrong".to_string()),
                },
            ),
            AppError::ErrorWrokWithDB => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "Bad request".to_string(),
                    details: Some("can`t get info by send parametrs".to_string()),
                },
            ),
        };
        (status, Json(error_response)).into_response()
    }
}

#[derive(Serialize)]
pub struct Message{
    pub message: String
}


// universal struct for sen normal json list data
#[derive(Serialize)]
pub struct Data<T>{
    pub data: T
}
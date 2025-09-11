use axum::{
    Json,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::Serialize;

// Определяем структуру для ответа об ошибке
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

// Определяем собственный тип ошибки
#[derive(Debug)]
pub enum AppError {
    InternalError,
    ErrorReadFile
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
            AppError::ErrorReadFile => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: "Resource not found".to_string(),
                    details: Some("possible lack of work services".to_string()),
                },
            ),
        };
        (status, Json(error_response)).into_response()
    }
}
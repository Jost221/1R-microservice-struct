use axum::{Json, extract::State, http::StatusCode, http::HeaderMap};
use crate::json_struct::*;
use crate::processing::AppState;
use crate::tokens;
use serde_json::json;
use std::sync::Arc;

pub async fn root_handler() -> Json<Message> {
    Json(Message {
        message: "Welcome to my microservice!".to_string(),
    })
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterPayload>,
) -> (StatusCode, Json<Message>) {
    match state.create_user(&payload.username, &payload.password).await {
        Ok(_) => (StatusCode::CREATED, Json(Message { message: "registered".into() })),
        Err(e) => {
            match e {
                crate::processing::UserError::Exists => (StatusCode::CONFLICT, Json(Message { message: "user exists".into() })),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(Message { message: "error".into() })),
            }
        }
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> (StatusCode, Json<TokenResponse>) {
    let mut status = StatusCode::OK;
    match state.verify_user(&payload.username, &payload.password).await {
        Ok(_) => {
            let (access, _jti_a, exp) = tokens::create_token(&state.jwt_secret, &payload.username, state.jwt_exp_seconds, "access").unwrap();
            let (refresh, jti_r, _exp_r) = tokens::create_token(&state.jwt_secret, &payload.username, state.refresh_exp_seconds, "refresh").unwrap();
            if let Err(_e) = state.store_refresh_jti(&jti_r, &payload.username, state.refresh_exp_seconds).await {
                status = StatusCode::INTERNAL_SERVER_ERROR;
            }
            (status, Json(TokenResponse { access_token: access, refresh_token: refresh, expires_in: exp }))
        }
        Err(_) => {
            status = StatusCode::UNAUTHORIZED;
            (status, Json(TokenResponse { access_token: "".into(), refresh_token: "".into(), expires_in: 0 }))
        }
    }
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshPayload>,
) -> (StatusCode, Json<serde_json::Value>) {
    match tokens::validate_token(&state.jwt_secret, &payload.refresh_token) {
        Ok(data) if data.claims.typ == "refresh" => {
            let jti = data.claims.jti.clone();
            match state.check_refresh_jti(&jti).await {
                Ok(Some(username)) => {
                    let _ = state.revoke_refresh_jti(&jti).await;
                    let (access, _aj, exp) = tokens::create_token(&state.jwt_secret, &username, state.jwt_exp_seconds, "access").unwrap();
                    let (refresh, rj, _rexp) = tokens::create_token(&state.jwt_secret, &username, state.refresh_exp_seconds, "refresh").unwrap();
                    let _ = state.store_refresh_jti(&rj, &username, state.refresh_exp_seconds).await;
                    (StatusCode::OK, Json(json!({"access_token": access, "refresh_token": refresh, "expires_in": exp})))
                }
                _ => (StatusCode::UNAUTHORIZED, Json(json!({"error":"invalid refresh token"}))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(json!({"error":"invalid refresh token"}))),
    }
}

pub async fn get_microservices() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"services": ["auth"], "status": "ok"})))
}

pub async fn get_current_user_roles(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UserRolesResponse>, (StatusCode, Json<Message>)> {
    // Извлекаем токен из заголовка
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| (
            StatusCode::UNAUTHORIZED,
            Json(Message { message: "Missing authorization header".into() })
        ))?;
    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(Message { message: "Invalid authorization format".into() })
        ));
    }

    let token = &auth_header[7..];
    // Валидируем токен и извлекаем username
    match tokens::validate_token(&state.jwt_secret, token) {
        Ok(token_data) if token_data.claims.typ == "access" => {
            let username = token_data.claims.sub;
            // Получаем роли пользователя
            match state.get_user_roles(&username).await {
                Ok(roles) => Ok(Json(UserRolesResponse { data: roles })),
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Message { message: "Failed to get user roles".into() })
                ))
            }
        }
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(Message { message: "Invalid or expired token".into() })
        ))
    }
}

/// Назначение ролей пользователю (только для админов)
pub async fn assign_roles(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<AssignRolePayload>,
) -> Result<Json<Message>, (StatusCode, Json<Message>)> {
    // Проверяем токен администратора
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| (
            StatusCode::UNAUTHORIZED,
            Json(Message { message: "Missing authorization header".into() })
        ))?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(Message { message: "Invalid authorization format".into() })
        ));
    }

    let token = &auth_header[7..];
    match tokens::validate_token(&state.jwt_secret, token) {
        Ok(token_data) if token_data.claims.typ == "access" => {
            let admin_username = token_data.claims.sub;
            // Проверяем, что у пользователя есть права админа
            match state.user_has_role(&admin_username, "admin").await {
                Ok(true) => {
                    // Назначаем роли
                    match state.assign_user_roles(&payload.username, payload.roles).await {
                        Ok(_) => Ok(Json(Message { message: "Roles assigned successfully".into() })),
                        Err(crate::processing::UserError::NotFound) => Err((
                            StatusCode::NOT_FOUND,
                            Json(Message { message: "User not found".into() })
                        )),
                        Err(_) => Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(Message { message: "Failed to assign roles".into() })
                        ))
                    }
                }
                _ => Err((
                    StatusCode::FORBIDDEN,
                    Json(Message { message: "Admin access required".into() })
                ))
            }
        }
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(Message { message: "Invalid or expired token".into() })
        ))
    }
}

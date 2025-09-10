
use axum::{Json, extract::State, http::StatusCode};
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

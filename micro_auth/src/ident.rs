use axum::{
    http::{Request, header, StatusCode},
    middleware::Next,
    response::Response,
    extract::State,
    body::Body,
};
use std::sync::Arc;

use crate::tokens;
use crate::processing::AppState;

pub async fn my_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,                            
) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            match tokens::validate_token(&state.jwt_secret, token) {
                Ok(data) if data.claims.typ == "access" => Ok(next.run(req).await),
                _ => Err(StatusCode::UNAUTHORIZED),
            }
        },
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

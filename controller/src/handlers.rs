use axum::{Json, extract::State};
use std::sync::Arc;

use crate::api_static::*;
use crate::processing;
use crate::sql_struct::*;

pub async fn root_handler(State(_app_state): State<Arc<AppState>>) -> Json<Message> {
    Json(Message {
        message: "Welcome to my microservice!".to_string()
    })
}

pub async fn get_users(State(app_state): State<Arc<AppState>>) -> Json<Data<Vec<User>>> {
    let users = processing::get_user::get_users(&app_state.sql_pool).await.unwrap();
    
    Json(Data{
        data: users
    })

}

pub async fn get_user_by_id(State(app_state): State<Arc<AppState>>, Path(id): Path<u64>) -> Json<User> {
    match processing::get_user::get_user_by_id(&app_state.sql_pool, &app_state.redis_pool, id).await {
       Ok(v) => Json(v),
       Err(e) => AppError::ErrorReadFile
    } 
}
    
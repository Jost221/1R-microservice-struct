use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, Debug, FromRow)]
pub struct User {
    pub name: Option<String>,
    pub id: i32,
}
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Users {
    pub username: Option<String>,
    pub role: Option<String>,
}
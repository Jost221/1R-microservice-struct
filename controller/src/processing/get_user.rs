use sqlx::{self, postgres, PgPool, Pool, Postgres};

use crate::sql_struct::Users;

async fn get_user(pool:Pool<Postgres>)->Result<(), sqlx::Error>{
    let users = sqlx::query_file_as!(Users,"src/requests/get_user.sql")
    .fetch_all(&pool)
    .await?;
    dbg!(users);
    Ok(())

}
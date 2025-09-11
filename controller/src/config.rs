use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub port: String,
    pub addres: String,
    pub sql_url: String,
    pub redis_url: String
}

pub struct JsonPath{
   pub path: String 
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();
        let port = env::var("PORT")?.parse().unwrap();
        let addres = env::var("ADDRESS")?.parse().unwrap();
        let sql_url = env::var("SQL_URL")?.parse().unwrap();
        let redis_url = env::var("Redis_URL")?.parse().unwrap();
        Ok(
            Self { 
                port, 
                addres, 
                sql_url,
                redis_url
            }
        )

    }

    pub fn address_to_string(&self) -> String {
        format!("{}:{}", self.addres, self.port)
    }
}
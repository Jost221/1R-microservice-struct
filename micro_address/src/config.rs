use dotenv::dotenv;
use std::env;

pub struct Config {
    pub port: u16,
    pub addres: String
}

pub struct JsonPath{
   pub path: String 
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();
        let port = env::var("PORT")?.parse().unwrap();
        let addres = env::var("ADDRESS")?.parse().unwrap();
        Ok(Self { port, addres })
    }

    pub fn address_to_string(&self) -> String {
        format!("{}:{}", self.addres, self.port)
    }
}

impl JsonPath{
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();
        Ok(Self { path: env::var("JSONPATH")?.parse().unwrap()})
    }

    pub fn to_string(&self) -> String{
        format!("{}", self.path)
    }
}
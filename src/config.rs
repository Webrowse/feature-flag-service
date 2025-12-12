use serde::Deserialize;
use dotenvy::dotenv;
use std::env;

#[derive(Deserialize)]
pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenv().is_ok();

        let port = env::var("PORT")
            .expect("PORT missing, it is required")
            .parse()
            .expect("PORT must be a valid u16 number");

        Self { port }
    }

    pub fn addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }
}
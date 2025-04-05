use std::{env, u16};
use lazy_static::lazy_static;

lazy_static!(
    pub static ref ADDRESS: String = set_address();
    pub static ref PORT: u16 = set_port();
    pub static ref DATABASE_URL: String = set_db_url();
    pub static ref SECRET: String = set_secret();
    pub static ref DIR_PATH: String = set_dir_path();
);

fn set_dir_path() -> String {
    dotenv::dotenv().ok();
    env::var("DIR_PATH").expect("No dir path set")
}


fn set_address() -> String {
    dotenv::dotenv().ok();
    env::var("ADDRESS").unwrap_or("127.0.0.1".to_string())
}

fn set_port() -> u16 {
    dotenv::dotenv().ok();
    env::var("PORT").unwrap_or("9090".to_owned())
        .parse::<u16>()
        .expect("Can't parse the port")
}

fn set_db_url() -> String {
    dotenv::dotenv().ok();
    env::var("DATABASE_URL")
        .expect("DATABASE_URL constant not found in env")
}

fn set_secret() -> String {
    dotenv::dotenv().ok();
    env::var("SECRET")
        .expect("Secret not found in env")
}

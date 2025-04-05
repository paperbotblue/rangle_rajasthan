use std::{error::Error, fmt::Display};
use actix_cors::Cors;
use actix_web::{middleware::Logger,  App, HttpServer};
use utils::app_state::AppState;

mod models;
mod routes;
mod services;
mod utils;

#[derive(Debug)]
struct MainError {
    message: String
}

impl Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl Error for MainError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None 
    }
    fn description(&self) -> &str {
        &self.message 
    }
    fn cause(&self) -> Option<&dyn Error> {
        self.source() 
    }

}

#[actix_web::main]
async fn main() -> Result<(), MainError> {

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    dotenv::dotenv().ok();
    env_logger::init();

    let port = (*utils::constants::PORT).clone();
    let address = (*utils::constants::ADDRESS).clone();
    let db = AppState::new().await;

    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .wrap(Logger::default())
            .configure(routes::auth_routes::config)
            .configure(routes::ticket_routes::config)
            .configure(routes::payment_routes::config)
            .configure(routes::ping_routes::config)
            .configure(routes::referral_routes::config)
            .configure(routes::bank_routes::config)
    })
    .bind(format!("{}:{}", address, port))
    .map_err(| err | MainError { message: err.to_string()}).unwrap()
    .run()
    .await
    .map_err(| err | MainError { message: err.to_string()})?;

    Ok(())

}


use actix_web::{
    middleware::from_fn, web::{self, scope}
};
use super::{handlers, middlewares};

pub fn config(config: &mut web::ServiceConfig)
{
    config
        .service(
            web::scope("/ticket")
                .wrap(from_fn(middlewares::auth_middleware::check_auth_middleware))
                .service(handlers::ticket_handlers::get_all_data)
                .service(handlers::ticket_handlers::get_qr_image)
                .service(handlers::ticket_handlers::verify_ticket)
                .service(handlers::ticket_handlers::update_ticket)
                .service(handlers::ticket_handlers::send_ticket_data))
        .service(
            web::scope("/mail")
                .service(handlers::ticket_handlers::generate_ticket_qr_code)
                .service(handlers::ticket_handlers::create_ticket)
        );
}

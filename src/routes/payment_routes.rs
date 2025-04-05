use actix_web::{
    web, middleware::from_fn
};
use super::{handlers, middlewares};

pub fn config(config: &mut web::ServiceConfig)
{
    config
        .service(
            web::scope("/payment")
                .service(handlers::payment_handlers::create_order)
        );
}

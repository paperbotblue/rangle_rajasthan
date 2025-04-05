use actix_web::{
    web, middleware::from_fn
};
use super::{handlers, middlewares};

pub fn config(config: &mut web::ServiceConfig)
{
    config
        .service(
            web::scope("/referral")
                .service(handlers::referral_handlers::create_referral)
                .service(handlers::referral_handlers::verify_referal)
                .service(handlers::referral_handlers::get_all_ref)
        );
}

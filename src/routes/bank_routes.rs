use actix_web::{middleware::from_fn, web};
use super::{handlers, middlewares};

pub fn config(config: &mut web::ServiceConfig)
{
    config
        .service(
            web::scope("/bank")
                .service(handlers::bank_handlers::register)
                .service(handlers::bank_handlers::login)
                .service(handlers::bank_handlers::verify_otp)
                .service(handlers::bank_handlers::send_mail)
        )
        .service(
            web::scope("/bank_auth")
                .wrap(from_fn(middlewares::auth_middleware::check_auth_middleware))
                .service(handlers::bank_handlers::get_accounts)
                .service(handlers::bank_handlers::get_history)
                .service(handlers::bank_handlers::get_cash_recharge)
                .service(handlers::bank_handlers::transactions)
                .service(handlers::bank_handlers::regharge_from_superadmin)
                .service(handlers::bank_handlers::get_recharge_amount)
                .service(handlers::bank_handlers::get_balance)
                .service(handlers::bank_handlers::add_money)
                .service(handlers::bank_handlers::transfer)
        );
}

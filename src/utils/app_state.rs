use actix_web::web::Data;
use mongodb::{
    Client, Collection
};
use crate::utils::constants;
use crate::models::{bank_models::{Bank, BankTransactions, Recharge, Otp, RechargeOffline}, user_model::User, ticket_model::Ticket, referal_code::ReferalCode};
use crate::services::db::Database;

pub struct AppState {
    pub db: Data<Database>
}

impl AppState {
    pub async fn new() -> Data<Database> {
        let database_url = (*constants::DATABASE_URL).clone();

        let client = match Client::with_uri_str(database_url).await {
            Ok(client) => client,
            Err(err) => {
                eprintln!("Failed to connect to MongoDB: {}", err);
                std::process::exit(1); // Stop execution if DB connection fails
            }
        };

        let db = client.database("ticket_db");

        let ticket: Collection<Ticket> = db.collection("ticket");
        let admin: Collection<User> = db.collection("admin");
        let referral: Collection<ReferalCode> = db.collection("referral_code");
        let bank: Collection<Bank> = db.collection("bank");
        let recharge: Collection<Recharge> = db.collection("recharge");
        let recharge_offline: Collection<RechargeOffline> = db.collection("recharge_offline");
        let bank_transactions: Collection<BankTransactions> = db.collection("bank_transactions");
        let otp: Collection<Otp> = db.collection("otp");

        Data::new(Database { referral,ticket, admin , bank, bank_transactions, recharge, otp, recharge_offline})
    }
}

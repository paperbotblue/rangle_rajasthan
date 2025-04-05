use mongodb::bson::oid::ObjectId;
use serde::{
    Serialize, Deserialize
};


#[derive(Serialize, Deserialize)]
pub struct Bank {
    pub _id: ObjectId,
    pub role: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
    pub balance: i64,
    pub qr_base64: String
}
#[derive(Serialize, Deserialize)]
pub struct BankTransactions {
    pub from_email: String,
    pub to_email: String,
    pub from: ObjectId,
    pub to: ObjectId,
    pub amount: i64
}
#[derive(Serialize, Deserialize)]
pub struct Otp {
    pub email: String,
    pub otp: String
}

#[derive(Serialize, Deserialize)]
pub struct Recharge {
    pub email: String,
    pub amount: i64
}
#[derive(Serialize, Deserialize)]
pub struct RechargeOffline {
    pub from: String,
    pub email: String,
    pub amount: i64
}



#[derive(Serialize, Deserialize)]
pub struct BankAccountCred{
    pub role: String,
    pub email: String,
    pub phone_number: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct ForgetPassword{
    pub email: String
}

#[derive(Serialize, Deserialize)]
pub struct VerifyOtp{
    pub email: String,
    pub new_password: String,
    pub otp: String
}

#[derive(Serialize, Deserialize)]
pub struct BankAccountCredLogin{
    pub email: String,
    pub password: String
}
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub to: String,
    pub amount: i64
}

#[derive(Serialize, Deserialize)]
pub struct AddMoney {
    pub amount: i64
}

#[derive(Serialize, Deserialize)]
pub struct Role {
    pub role: String
}


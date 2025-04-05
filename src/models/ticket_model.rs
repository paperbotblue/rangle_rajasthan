use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub ref_code: String,
    pub category: String,
    pub additional_humans: i32,
    pub children: i32,
    pub used: bool,
    pub amount: f64,
    pub is_online: bool
}
// when needed regenerate ticket_base64 using ticket_uuid
#[derive(Debug, Serialize, Deserialize)]
pub struct TicketRequest {
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub ref_code: String,
    pub category: String,
    pub additional_humans: i32,
    pub children: i32,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketResponse  {
    pub ticket_base64: String,
}


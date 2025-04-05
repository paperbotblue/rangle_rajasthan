use mongodb::bson::oid::ObjectId;
use serde::{
    Serialize,Deserialize
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferalCode {
    pub referral: String,
    pub promoterName: String,
    pub discount: String
} 

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferralRequest {
    pub referrals: Vec<ReferalCode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferralRequestCheck {
    pub referrals: String,
}

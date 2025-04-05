use crate::models::referal_code::{ReferralRequest, ReferralRequestCheck};
use crate::services::db::Database;
use crate::utils::api_responses::ApiResponse;
use actix_web::{
    post, web, get 
};
use futures_util::StreamExt;
use mongodb::bson;
#[post("/verify")]
async fn verify_referal(
    app_state: web::Data<Database>,
    request: web::Json<ReferralRequestCheck>,
) -> Result<ApiResponse, ApiResponse> {
    let referral_code = request.referrals.clone();

    let query = bson::doc! { "referral": &referral_code };
    match app_state.referral.find_one(query).await {
        Ok(Some(referral_data)) => {
            let data = serde_json::json!({
                "discount" : referral_data.discount,
            });
            Ok(ApiResponse::json(200, data))
        }
        Ok(None) => {
            Ok(ApiResponse::text(404, "Referral code not found".to_owned()))
        }
        Err(err) => {
            Err(ApiResponse::text(500, format!("Database error: {}", err)))
        }
    }
}
#[post("")]
async fn create_referral(
    app_state: web::Data<Database>,
    request: web::Json<ReferralRequest>,
) -> Result<ApiResponse, ApiResponse> {
    let _ = app_state.referral.drop().await;
    let referrals = request.into_inner().referrals;

    if let Err(err) = app_state.referral.insert_many(referrals).await {
        return Err(ApiResponse::text(500, err.to_string()));
    }

    Ok(ApiResponse::text(200, "success".to_owned()))
}

#[get("")]
async fn get_all_ref(
    app_state: web::Data<Database>,
) -> Result<ApiResponse, ApiResponse> {
    let filter = bson::doc! {};
    let cursor = match app_state.referral.find(filter).await {
        Ok(cursor) => cursor,
        Err(err) => return Err(ApiResponse::text(500, format!("Database query failed: {}", err))),
    };

    let mut referrals = Vec::new();
    let mut cursor = cursor;

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(ticket) => referrals.push(ticket),
            Err(err) => return Err(ApiResponse::text(500, format!("Failed to parse ticket: {}", err))),
        }
    }

    // Return tickets as JSON response
    Ok(ApiResponse::json(200, serde_json::json!({ "referrals": referrals})))
}





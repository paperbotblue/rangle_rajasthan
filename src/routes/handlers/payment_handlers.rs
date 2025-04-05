use actix_web::{web, post};
use mongodb::bson;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::utils::api_responses::ApiResponse;
use crate::services::db::Database;


#[derive(Serialize, Deserialize)]
struct OrderRequest {
    email: String,
    amount: i64,
}

#[derive(Serialize, Deserialize)]
struct RazorpayOrderResponse {
    id: String,
    amount: u64,
    currency: String,
}


#[post("/{amount}/{email}")]
async fn create_order(
    app_state: web::Data<Database>,
    amount: web::Path<OrderRequest>,
) -> Result<ApiResponse, ApiResponse> {
    match app_state.ticket.find_one(bson::doc! {"email": amount.email.clone()}).await {
        Ok(Some(user)) => Err(ApiResponse::text(400, "user already exist".to_string()))?,
        Ok(None) => {}
        Err(err) => Err(ApiResponse::text(500, err.to_string()))?
    }

    //fake keys
    let razorpay_key_id = "sdufbsebfiseui987es98f79";
    let razorpay_key_secret = "sjdkdjfkls8sd7f98sdf9ds8";

    let client = Client::new();
    let order_url = "https://api.razorpay.com/v1/orders";

    let order_request = serde_json::json!({
        "amount": amount.amount * 100,
        "currency": "INR",
        "receipt": "order_receipt_1",
    });

    let response = client
        .post(order_url)
        .basic_auth(&razorpay_key_id, Some(&razorpay_key_secret))
        .body(order_request.to_string())
        .send()
        .await;
    match response {
        Ok(res) => {
            let status = res.status();
            if status.is_success() {
                let body = res.text().await.map_err(|e| {
                    ApiResponse::json(500, serde_json::json!({"status": "failed", "data": "not good"}))
                })?;
                Ok(ApiResponse::json(200, serde_json::json!({"status": "success", "data": body})))
            } else {
                Err(ApiResponse::json(
                    status.into(),
                    serde_json::json!({"status": "failed", "data": "not good"}),
                ))
            }
        }
        Err(err) => {
            Err(ApiResponse::json(
                500,
                serde_json::json!({"status": "failed", "data": "not good"}),
            ))
        }
    }
}


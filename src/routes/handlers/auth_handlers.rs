use actix_web::{
    post, 
    web::Json, web::Data
};
use sha256::digest;
use mongodb::bson::doc;
use serde_json::json;
use crate::models::user_model::{
    User, UserLogin, UserRegister
};
use crate::utils::jwt::encode_jwt;
use crate::services::db::Database;
use crate::utils::api_responses::ApiResponse;

#[post("/register")]
pub async fn register(
    app_state: Data<Database>,
    request: Json<UserRegister>
) -> Result<ApiResponse, ApiResponse> {

    let filter = doc! { "email": &request.email };
    match app_state.admin.find_one(filter).await {
        Ok(Some(_)) => return Err(ApiResponse::text(400, "Email is already registered".to_string())),
        Ok(None) => {} 
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    }

    let admin = User::try_from(request.into_inner())
        .map_err(|err| ApiResponse::text(500, format!("Conversion error: {}", err)))?;

    app_state.create_admin(admin).await
        .map_err(|err| ApiResponse::text(500, format!("Database error: {}", err)))?;

    Ok(ApiResponse::text(201, "User registered successfully".to_string()))
}


#[post("/login")]
pub async fn login(
    app_state: Data<Database>,
    request: Json<UserLogin>
) -> Result<ApiResponse, ApiResponse> {
    let hashed_password = digest(&request.password);
    let filter = doc! { "email": &request.email , "password": hashed_password};
    match app_state.admin.find_one(filter).await {
        Ok(Some(user)) => {
            let is_verified = user.verified;
            if !is_verified {
                return Err(ApiResponse::text(400, "admin is not validated".to_owned()))?
            }
        },
        Ok(None) => return Err(ApiResponse::text(401, "Invalid email or password".to_owned())),
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err)))?,
    }

    let admin = app_state    
        .verify_admin(
           request.0 
        ).await.map_err(| err | ApiResponse::json(400, serde_json::json!({"status": "failed"})))?;

    let token = encode_jwt(admin.email, admin._id)
        .map_err(| err | ApiResponse::text(500, err.to_string()))?;
    let res = json!({
        "token": token,
        "role": admin.role
    });
    Ok(ApiResponse::json(200, res))
}



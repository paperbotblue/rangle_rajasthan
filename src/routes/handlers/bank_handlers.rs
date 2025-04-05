use actix_web::{
    get,
    post, 
    web::Json, web::Data
};
use sha256::digest;
use futures_util::StreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::doc;
use serde_json::json;
use crate::utils::jwt::{encode_jwt, Claims};
use crate::services::db::Database;
use crate::utils::api_responses::ApiResponse;
use crate::models::bank_models::{
    AddMoney, Bank, BankAccountCred, BankAccountCredLogin, BankTransactions, ForgetPassword, Otp, Recharge, RechargeOffline, Role, Transaction, VerifyOtp
};
use qrcode::QrCode;
use image::Luma;
use base64::{engine::general_purpose, Engine};
use std::io::Cursor;
use std::str::FromStr;
use lettre::message::{Mailbox, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};



#[post("/register")]
pub async fn register(
    app_state: Data<Database>,
    request: Json<BankAccountCred>
) -> Result<ApiResponse, ApiResponse> {

    let filter = doc! { "email": &request.email };
    match app_state.bank.find_one(filter).await {
        Ok(Some(_)) => return Err(ApiResponse::text(400, "Email is already registered".to_string())),
        Ok(None) => {
            let id = ObjectId::new();
            let bank_account = Bank {
                _id: id,
                role: request.role.clone(),
                email: request.email.clone(),
                password: digest(request.password.clone()),
                phone_number: request.phone_number.clone(),
                balance: 0,
                qr_base64: generate_base64_qr(&id.to_string())
            };
            app_state.bank.insert_one(bank_account)
                .await
                .map_err(| err | ApiResponse::text(500, err.to_string()))?;
            app_state.otp.insert_one(Otp {email: request.email.clone(), otp: "".to_string()})
                .await
                .map_err(| err | ApiResponse::text(500, err.to_string()))?;
            Ok(ApiResponse::text(201, "User registered successfully".to_string()))

        } 
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    }
}
#[post("/recharge_from_superadmin")]
pub async fn regharge_from_superadmin(
    app_state: Data<Database>,
    claims: Claims,
    request: Json<Recharge>,
) -> Result<ApiResponse, ApiResponse> {
    match app_state.bank.find_one(doc! {"email": &request.email }).await {
        Ok(Some(account)) => {
            app_state.recharge_offline.insert_one(RechargeOffline {from: claims.email, email: account.email.clone(), amount: request.amount})
                .await.map_err(|err| ApiResponse::text(500, err.to_string()))?;

            match app_state.bank
                .update_one(
                    doc! {"email": &account.email},
                    doc! {"$inc": {"balance": &request.amount}},
                )
                .await
            {
                Ok(_) => Ok(ApiResponse::text(200, "Balance updated successfully".to_owned())),
                Err(err) => Err(ApiResponse::text(500, err.to_string())),
            }
        }
        Ok(None) => Err(ApiResponse::text(400, "Email not found".to_owned())),
        Err(err) => Err(ApiResponse::text(500, err.to_string())),
    }
}
#[get("/cash_recharge")]
pub async fn get_cash_recharge(
    app_state: Data<Database>,
    claims: Claims,
) -> Result<ApiResponse, ApiResponse> {
    let cursor = app_state.recharge_offline.find(doc! {}).await;

    match cursor {
        Ok(mut cursor) => {
            let mut results = Vec::new();
            while let Some(doc) = cursor.next().await {
                match doc {
                    Ok(record) => results.push(record),
                    Err(err) => return Err(ApiResponse::text(500, err.to_string())),
                }
            }
            results.reverse();
            Ok(ApiResponse::json(200, serde_json::json!({ "data": results })))
        }
        Err(err) => Err(ApiResponse::text(500, err.to_string())),
    }
}

#[post("/login")]
pub async fn login(
    app_state: Data<Database>,
    request: Json<BankAccountCredLogin>
) -> Result<ApiResponse, ApiResponse> {
    let hashed_password = digest(&request.password);
    let filter = doc! { "email": &request.email , "password": hashed_password};
    match app_state.bank.find_one(filter).await {
        Ok(Some(user)) => {
            let token = encode_jwt(user.email.clone(), user._id)
                .map_err(| err | ApiResponse::text(500, err.to_string()))?;
            let res = json!({
                "token": token,
                "role": user.role,
                "email": user.email,
                "phoneNumber": user.phone_number
            });
            Ok(ApiResponse::json(200, res))
        },
        Ok(None) => return Err(ApiResponse::text(401, "Invalid email or password".to_owned())),
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err)))?,
    }
}

#[post("/forget")]
pub async fn send_mail(
    app_state: Data<Database>,
    request: Json<ForgetPassword>
) -> Result<ApiResponse, ApiResponse> {
    let user = match app_state.bank.find_one(doc! {"email": &request.email}).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(ApiResponse::text(400, "email does not exist".to_owned())),
        Err(err) => return Err(ApiResponse::text(500, err.to_string()))                                 
    };

    let otp = send_otp_email(request.email.as_str()).await.map_err(|err| ApiResponse::text(500, err.to_string()))?;
    app_state.otp
    .update_one(
        doc! { "email": &request.email },  // Query filter
        doc! { "$set": { "otp": otp.clone() } },  // Update operation
    )
    .await
    .map_err(|err| ApiResponse::text(500, err.to_string()))?;
    Ok(ApiResponse::json(200, serde_json::json!({"status": "success"})))
}

#[post("/verify")]
pub async fn verify_otp(
    app_state: Data<Database>,
    request: Json<VerifyOtp>
) -> Result<ApiResponse, ApiResponse> {
    let account = match app_state.otp.find_one(doc! {"email": &request.email}).await {
        Ok(Some(success)) => {
            if request.otp == success.otp {
                app_state.bank
                    .update_one(
                        doc! { "email": &request.email },  // Query filter
                        doc! { "$set": { "password": digest(&request.new_password) } },  // Correct update syntax
                    )
                    .await
                    .map_err(|err| ApiResponse::text(500, err.to_string()))?;
                return Ok(ApiResponse::text(200, "success".to_owned()));
            } else {
                return Ok(ApiResponse::text(401, "invalid otp".to_owned()));
            }
        }
        Ok(None) => return Ok(ApiResponse::text(400, "email not found".to_owned())),
        Err(err) => return Err(ApiResponse::text(500, err.to_string())),
    };
}

#[post("/history")]
pub async fn get_history(
    app_state: Data<Database>,
    claims: Claims,
    request: Json<Role>,
) -> Result<ApiResponse, ApiResponse> {
    let filter = match request.role.as_str() {
        "user" => doc! { "from": &claims._id },
        "seller" => doc! { "to": &claims._id },
        _ => doc! {},
    };

    let mut cursor = match app_state.bank_transactions.find(filter).await {
        Ok(cursor) => cursor,
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    };

    let mut history = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(transaction) => history.push(transaction),
            Err(err) => return Err(ApiResponse::text(500, format!("Error processing transaction: {}", err))),
        }
    }

    history.reverse();

    let res = serde_json::json!({ "history": history });

    Ok(ApiResponse::json(200, res))
}
#[get("/accounts")]
pub async fn get_accounts(
    app_state: Data<Database>,
    claims: Claims
) -> Result<ApiResponse, ApiResponse> {
    let filter = doc! {};
    let mut cursor = match app_state.bank.find(filter).await {
        Ok(cursor) => cursor,
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    };

    let mut history = Vec::new();
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(transaction) => history.push(transaction),
            Err(err) => return Err(ApiResponse::text(500, format!("Error processing transaction: {}", err))),
        }
    }

    let res = serde_json::json!({ "history": history });

    Ok(ApiResponse::json(200, res))
}

#[get("/balance")]
pub async fn get_balance(
    app_state: Data<Database>,
    claims: Claims
) -> Result<ApiResponse, ApiResponse> {
    let filter = doc! {"_id": claims._id};
    match app_state.bank.find_one(filter).await {
        Ok(Some(user)) => {
            let res = serde_json::json!({ "balance": user.balance, "base64_qr": user.qr_base64});
            return Ok(ApiResponse::json(200, res));
        }
        Ok(None) => {
            return Err(ApiResponse::text(400, "not found user".to_string()));
        }
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    };
}
#[get("/recharge_amount")]
pub async fn get_recharge_amount(
    app_state: Data<Database>,
    claims: Claims,
) -> Result<ApiResponse, ApiResponse> {
    let mut cursor = match app_state.recharge.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(err) => return Err(ApiResponse::text(500, err.to_string())),
    };

    let mut data = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(recharge) => data.push(recharge),
            Err(err) => return Err(ApiResponse::text(500, format!("Error processing recharge: {}", err))),
        }
    }
    data.reverse();

    Ok(ApiResponse::json(200, serde_json::json!({ "data": data })))
}

#[post("/add_money")]
pub async fn add_money(
    app_state: Data<Database>,
    claims: Claims,
    request: Json<AddMoney>
) -> Result<ApiResponse, ApiResponse> {
    if request.amount <= 0 {
        return Err(ApiResponse::text(400, "Invalid amount".to_owned()));
    }

    let mut amount = request.amount;
    if request.amount == 1000 {
        amount = 1200;
    } else if request.amount == 2000 {
        amount = 2500;
    } else if request.amount == 3000 {
        amount = 4000;
    } 

    let user_filter = doc! { "_id": &claims._id };
    
    let user_account = match app_state.bank.find_one(user_filter.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(ApiResponse::text(401, "User not found".to_owned())),
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    };

    app_state.recharge.insert_one(Recharge {email: claims.email, amount: request.amount})
        .await
        .map_err(| err | ApiResponse::text(500, err.to_string()))?;
    
    let update_doc = doc! { "$inc": { "balance": amount } };
    match app_state.bank.update_one(user_filter, update_doc).await {
        Ok(_) => Ok(ApiResponse::text(200, "Money added successfully".to_owned())),
        Err(_) => Err(ApiResponse::text(500, "Database error".to_owned())),
    }
}

#[get("/transactions")]
pub async fn transactions(
    app_state: Data<Database>,
    claims: Claims,
) -> Result<ApiResponse, ApiResponse> {
    let mut cursor = app_state
        .bank_transactions
        .find(doc! {}) // Second argument is options (None for default)
        .await
        .map_err(|err| ApiResponse::text(500, err.to_string()))?;

    let mut transactions = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(transaction) => transactions.push(transaction),
            Err(err) => return Err(ApiResponse::text(500, err.to_string())),
        }
    }

    transactions.reverse();

    Ok(ApiResponse::json(200, serde_json::json!({
        "data": transactions
    })))
}





#[post("/transfer")]
pub async fn transfer(
    app_state: Data<Database>,
    claims: Claims,
    request: Json<Transaction>,
) -> Result<ApiResponse, ApiResponse> {
    let sender_id = claims._id.clone();
    let recipient_id = match ObjectId::from_str(request.to.as_str()) {
        Ok(id) => id,
        Err(_) => return Err(ApiResponse::text(400, "Invalid recipient ID".to_owned())),
    };

    let amount = request.amount;

    if amount <= 0 {
        return Err(ApiResponse::text(400, "Invalid amount".to_owned()));
    }

    let sender_filter = doc! { "_id": &sender_id};
    let recipient_filter = doc! { "_id": &recipient_id};

    let sender = match app_state.bank.find_one(sender_filter.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(ApiResponse::text(401, "Invalid sender".to_owned())),
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    };

    if sender.balance < amount {
        return Err(ApiResponse::text(400, "Insufficient balance".to_owned()));
    }

    let recipient = match app_state.bank.find_one(recipient_filter.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(ApiResponse::text(401, "Invalid recipient".to_owned())),
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    };
    let transaction = BankTransactions {
        from_email: sender.email,
        to_email: recipient.email,
        from: sender_id,
        to: recipient_id,
        amount
    };
    app_state.bank_transactions.insert_one(transaction)
        .await
        .map_err( |err | ApiResponse::text(500 , err.to_string()))?;

    let sender_update = doc! { "$set": { "balance": sender.balance - amount } };
    let recipient_update = doc! { "$set": { "balance": recipient.balance + amount } };

    let sender_update_result = app_state.bank.update_one(sender_filter, sender_update).await;
    let recipient_update_result = app_state.bank.update_one(recipient_filter, recipient_update).await;

    if sender_update_result.is_err() || recipient_update_result.is_err() {
        return Err(ApiResponse::text(500, "Transaction failed".to_owned()));
    }

    Ok(ApiResponse::text(200, "Transfer successful".to_owned()))
}

fn generate_base64_qr(data: &str) -> String {
    let code = QrCode::new(data.as_bytes()).unwrap();
    let image = code.render::<Luma<u8>>().build();
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageOutputFormat::Png).unwrap();
    let base64_string = general_purpose::STANDARD.encode(buffer.get_ref());
    format!("data:image/png;base64,{}", base64_string)
}

pub async fn send_otp_email(to_email: &str) -> Result<String, lettre::transport::smtp::Error> {
    let smtp_server: &str = "smtp.hostinger.com";
    let smtp_username: &str = "brightwings@ranglerajasthan.in";
    let smtp_password: &str = "BrightWings@9783";
    let from_email: &str = "brightwings@ranglerajasthan.in";

    let subject = "🔐 Your One-Time Password (OTP) for Verification";

    // Generate a 6-digit OTP
    let otp: u32 = rand::random_range(100_000..999_999);

    let body = format!(
        r#"
        <html>
        <head>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    line-height: 1.6;
                    color: #333;
                }}
                h1 {{
                    color: #e74c3c;
                    text-align: center;
                }}
                .otp-box {{
                    font-size: 24px;
                    font-weight: bold;
                    background: #f3f3f3;
                    padding: 10px;
                    display: inline-block;
                    border-radius: 5px;
                    margin: 10px auto;
                }}
                p {{
                    font-size: 16px;
                }}
            </style>
        </head>
        <body>
            <h1>🔐 Your One-Time Password (OTP)</h1>
            <p>Dear User,</p>
            <p>Your OTP for verification is:</p>
            <div class="otp-box">{otp}</div>
            <p>This OTP is valid for 10 minutes. Do not share it with anyone.</p>
            <p>If you did not request this, please ignore this email.</p>
            <p>Best Regards,<br><strong>Bright Wings Travel & Tourism</strong></p>
        </body>
        </html>
        "#,
        otp = otp
    );

    let email = Message::builder()
        .from(Mailbox::new(None, from_email.parse().unwrap()))
        .to(Mailbox::new(None, to_email.parse().unwrap()))
        .subject(subject)
        .singlepart(SinglePart::html(body))
        .unwrap();

    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
    let mailer = SmtpTransport::relay(smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)?;

    // Return the OTP so it can be verified later
    Ok(otp.to_string())
}


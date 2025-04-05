use actix_web::{
    get, post, put, web
};
use sha256::digest;
use futures_util::StreamExt;
use crate::models::ticket_model::TicketRequest;
use crate::models::bank_models::Otp;
use crate::models::bank_models::Bank;
use crate::utils::api_responses::ApiResponse;
use crate::utils::jwt::Claims;
use mongodb::bson::oid::ObjectId;
use mongodb::bson;
use crate::models::ticket_model::Ticket;
use crate::services::db::Database;
use image::{
    Luma, ImageOutputFormat, ImageFormat, RgbaImage
};
use crate::utils;
use lettre::message::{header, Mailbox, Message ,MultiPart, SinglePart, Attachment};
use lettre::{SmtpTransport, Transport, transport::smtp::authentication::Credentials};
use qrcode::QrCode;
use std::fmt::format;
use std::io::Cursor;
use std::fs;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Read;

#[post("")]
async fn create_ticket(
    app_state: web::Data<Database>,
    register_json: web::Json<TicketRequest>,
) -> ApiResponse {
    let dir_path = (*utils::constants::DIR_PATH).clone();
    let qr_folder: String = format!("{}/qr_temp",dir_path);
    let qr_output: String = format!("{}/qr_temp/front_side",dir_path);

    let ticket = generate_ticket(register_json.into_inner());

    let _ = app_state.create_ticket(&ticket).await;

    let qr_path = format!("{}/image{}.png", qr_folder, ticket.id);
    let output_path = format!("{}/ticket_{}.png", qr_output, ticket.id);

    if let Err(err) = generate_qr_image(ticket.id, qr_path.clone()).await {
        return ApiResponse::json(500, serde_json::json!({
            "resr": format!("Err {} \n {} \n {}", err, qr_path, output_path)
        }));
    }

    if let Err(err) = generate_ticket_image(&qr_path, &output_path) {
        return ApiResponse::json(500, serde_json::json!({
            "res": format!("Err {} \n {} \n {}", err, qr_path, output_path)
        }));
    }

     //let _ = app_state.otp.insert_one(Otp {email: ticket.email.clone(), otp: "".to_string()})
     //   .await
     //   .map_err(| err | ApiResponse::text(500, err.to_string()));
    
    //if ticket.ref_code != "" {
    //    let referal = match app_state.referral.find_one(bson::doc! {"referral": ticket.ref_code}).await {
    //        Ok(Some(referal)) => referal,
    //        Ok(None) => return ApiResponse::text(400, "Invalid referal code".to_owned()),
    //        Err(_) => return ApiResponse::text(500, "Internal server error".to_owned())
    //    };
    //
    //    if referal.discount.parse::<i32>().unwrap() > 49 {
    //        let _ = create_bank_zero_account(ticket.username.clone(), ticket.email.clone(),ticket.phone_number.clone(), ticket.category.clone(), app_state).await;
    //    } else {
    //        let _ = create_bank_account(ticket.username.clone(), ticket.email.clone(),ticket.phone_number.clone(), ticket.category, app_state).await;
    //    }
    //
    //} else {
    //    let _ = create_bank_zero_account(ticket.username.clone(), ticket.email.clone(),ticket.phone_number.clone(), ticket.category.clone(), app_state).await;
    //}

    let mut file = match fs::File::open(&output_path) {
        Ok(f) => f,
        Err(_) => return ApiResponse::text(500, "Failed to open ticket image".to_owned()),
    };

    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() {
        return ApiResponse::text(500, "Failed to read ticket image".to_owned());
    }

    let base64_string = STANDARD.encode(&buffer);
    let base64_image = format!("data:image/png;base64,{}", base64_string);
    let _ = send_email(ticket.email.to_lowercase().as_str(), &output_path).await;

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&qr_path);
    ApiResponse::json(200, serde_json::json!({ "ticket": base64_image }))

}



#[put("/{id}")]
async fn update_ticket(
    app_state: web::Data<Database>,
    ticket_uuid: web::Path<String>,
    claim: Claims,
    register_json: web::Json<TicketRequest>,
) -> Result<ApiResponse, ApiResponse> {
    let object_id = ObjectId::parse_str(ticket_uuid.clone()).map_err(|err| {
        ApiResponse::text(400, format!("Invalid ticket ID: {}", err))
    })?;

    let ticket_state = app_state
        .find_ticket(object_id)
        .await
        .map_err(|err| {
            ApiResponse::json(500, serde_json::json!({"status": err.to_string()}))
        })?;

    match ticket_state {
        Some(ticket) => {
            let update_doc = bson::doc! {
                "$set": {
                "username": register_json.username.clone(),
                "email": register_json.email.clone(),
                "phone_number": register_json.phone_number.clone(),
                "ref_code": register_json.ref_code.clone(),
                "category": register_json.category.clone(),
                "additional_humans": register_json.additional_humans,
                "children": register_json.children,
                "used": false,
                "amount": register_json.amount
            }
        };

            let query = bson::doc! {"_id": object_id};
            app_state
                .ticket
                .update_one(query, update_doc)
                .await
                .map_err(|err| ApiResponse::text(500, err.to_string()))?;
            //let _ = app_state.otp.insert_one(Otp {email: ticket.email.clone(), otp: "".to_string()})
            //    .await
            //    .map_err(| err | ApiResponse::text(500, err.to_string()));
            //
            //
            //if ticket.ref_code != "" {
            //    let referal = match app_state.referral.find_one(bson::doc! {"ref_code": ticket.ref_code}).await {
            //        Ok(Some(referal)) => referal,
            //        Ok(None) => return Ok(ApiResponse::text(400, "Invalid referal code".to_owned())),
            //        Err(_) => return Ok(ApiResponse::text(500, "Internal server error".to_owned()))
            //    };
            //
            //    if referal.discount.parse::<i32>().unwrap() > 49 {
            //        let _ = create_bank_zero_account(ticket.username.clone(), ticket.email.clone(),ticket.phone_number.clone(), ticket.category.clone(), app_state).await;
            //    } else {
            //        let _ = create_bank_account(ticket.username.clone(), ticket.email.clone(),ticket.phone_number.clone(), ticket.category, app_state).await;
            //    }
            //
            //} else {
            //    let _ = create_bank_zero_account(ticket.username.clone(), ticket.email.clone(),ticket.phone_number.clone(), ticket.category.clone(), app_state).await;
            //}          

            Ok(ApiResponse::json(200, serde_json::json!({"status": "Ticket updated successfully"})))
        }
        None => {
            Ok(ApiResponse::json(404, serde_json::json!({"status": "Ticket not found"})))
        }
    }
}

#[get("")]
async fn get_all_data(
    app_state: web::Data<Database>,
    claim: Claims,

) -> Result<ApiResponse, ApiResponse> {
    // Reference the collection
    let collection = app_state.ticket.clone();

    // Fetch documents where email is not an empty string
    let filter = bson::doc! { "email": { "$ne": "" } }; // Filter for non-empty email
    let cursor = match collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(err) => return Err(ApiResponse::text(500, format!("Database query failed: {}", err))),
    };

    let mut tickets = Vec::new();
    let mut cursor = cursor;

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(ticket) => tickets.push(ticket),
            Err(err) => return Err(ApiResponse::text(500, format!("Failed to parse ticket: {}", err))),
        }
    }

    // Return tickets as JSON response
    Ok(ApiResponse::json(200, serde_json::json!({ "tickets": tickets })))
}


#[get("/{id}")]
async fn send_ticket_data(
    app_state: web::Data<Database>,
    ticket_id: web::Path<String>,
    claim: Claims
) -> Result<ApiResponse, ApiResponse> {
    let object_id = ObjectId::parse_str(ticket_id.clone()).map_err(|err| {
        ApiResponse::text(400, format!("Invalid ticket ID: {}", err))
    })?;
    let ticket_state = app_state
        .find_ticket(object_id)
        .await
        .map_err(|err| {
            ApiResponse::json(500, serde_json::json!({"status": err.to_string()}))
        })?;
    match ticket_state {
        Some(ticket) => {

            let res = serde_json::json!({
                "ticket": ticket
            }); 
            return Ok(ApiResponse::json(200, res))
        }
        None => return Ok(ApiResponse::json(400,serde_json::json!({"status": "Ticket is invalid"})))
    }
}
#[get("/verify/{ticket_uuid}")]
async fn verify_ticket(
    app_state: web::Data<Database>,
    ticket_uuid: web::Path<String>,
    claim: Claims

) -> Result<ApiResponse, ApiResponse> {
    let object_id = ObjectId::parse_str(ticket_uuid.clone()).map_err(
        | err | ApiResponse::text(500, err.to_string())
    )?;
    let ticket_state = app_state
        .find_ticket(object_id)
        .await
        .map_err(|err| {
            ApiResponse::json(500, serde_json::json!({"status": err.to_string()}))
        })?;

    match ticket_state {
        Some(ticket) => {
            if ticket.used {
                return Ok(ApiResponse::json(400, serde_json::json!({"status": "Ticket is already used", "ticket": ticket})));
            }
            let query = bson::doc! {"_id": ticket.id};
            let update = bson::doc! { "$set": { "used": true } };
            app_state
                .ticket
                .update_one(query, update)
                .await
                .map_err(|err| {
                    ApiResponse::json(500, serde_json::json!({"status": err.to_string()}))
                })?;
            let res = serde_json::json!({
                "status": "welcome",
                "ticket": ticket 
            }); 
            Ok(ApiResponse::json(200, res))
        }
        None => Ok(ApiResponse::json(
            400,
            serde_json::json!({"status": "Ticket is invalid"}),
        )),
    }
}

#[get("/get_qr/{id}")]
async fn get_qr_image(
    id: web::Path<String>,
    claim: Claims

) -> Result<ApiResponse, ApiResponse> {
    let code = QrCode::new(id.clone())
        .map_err(|err| ApiResponse::text(500, err.to_string()))?;

    let image = code.render::<Luma<u8>>().build();
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageOutputFormat::Png)
        .map_err(|err| ApiResponse::text(500, err.to_string()))?;
    let q_id = ObjectId::parse_str(id.into_inner()).expect("Invalid ObjectId format");
    if let Ok(base64_image) = generate_qr_base64(q_id).map_err(|err| ApiResponse::text(500, err.to_string())) {
        let data = serde_json::json!({ "qr": base64_image });
        Ok(ApiResponse::json(200, data))
    } else {
        Err(ApiResponse::text(404, "QR code generation failed".to_owned()))
    }

}

#[get("/generate/{quantity}")]
async fn generate_ticket_qr_code(
    app_state: web::Data<Database>,
    quantity: web::Path<i32>,
) -> Result<ApiResponse, ApiResponse> {
    for i in 0..*quantity {
        let ticket_uuid = ObjectId::new();
        let ticket = Ticket {
            id: ticket_uuid,
            username: "".to_string(),
            email: "".to_string(),
            phone_number: "".to_string(),
            ref_code: "".to_string(),
            category: "".to_string(),
            additional_humans: 0,
            children: 0,
            used: false,
            amount: 0.0,
            is_online: false
        };

        if let Err(err) = app_state.ticket.insert_one(ticket).await {
            return Err(ApiResponse::text(500, format!("Failed to insert ticket: {}", err)));
        }

        let src = format!("qrs/image{}.png", i);
        if let Err(err) = generate_qr_image(ticket_uuid, src.clone()).await {
            return Err(ApiResponse::text(500, format!("Failed to generate QR code: {}", err)));
        }
    }

    // Return success response
    Ok(ApiResponse::text(200, format!("Created {} tickets and QR codes", quantity)))
}

pub async fn create_bank_account(
    username: String,
    email: String,
    phone_number: String,
    ticket_cateogery: String,
    app_state: web::Data<Database>

) -> Result<ApiResponse, ApiResponse> {
    let mut money = 0;
    if ticket_cateogery == "individual" {
        money = 250;
    } else if  ticket_cateogery == "couple" {
        money = 350;
    } else if ticket_cateogery == "family" {
        money = 500;
    } 
    let filter = bson::doc! { "email": email.clone() };
    match app_state.bank.find_one(filter).await {
        Ok(Some(_)) => return Err(ApiResponse::text(400, "Email is already registered".to_string())),
        Ok(None) => {
            let id = ObjectId::new();
            let bank_account = Bank {
                _id: id,
                role: "user".to_string(),
                email: email.clone(),
                password: digest(format!("{}@{}", username, phone_number)),
                phone_number,
                balance: money,
                qr_base64: generate_qr_base64(id).expect("unable to add")
            };
            app_state.bank.insert_one(bank_account)
                .await
                .map_err(| err | ApiResponse::text(500, err.to_string()))?;
            Ok(ApiResponse::text(201, "User registered successfully".to_string()))
        } 
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    }
}

pub async fn create_bank_zero_account(
    username: String,
    email: String,
    phone_number: String,
    ticket_cateogery: String,
    app_state: web::Data<Database>

) -> Result<ApiResponse, ApiResponse> {
    let mut money = 0;
    if ticket_cateogery == "individual" {
        money = 250;
    } else if  ticket_cateogery == "couple" {
        money = 350;
    } else if ticket_cateogery == "family" {
        money = 500;
    } 
    let filter = bson::doc! { "email": email.clone() };
    match app_state.bank.find_one(filter).await {
        Ok(Some(_)) => return Err(ApiResponse::text(400, "Email is already registered".to_string())),
        Ok(None) => {
            let id = ObjectId::new();
            let bank_account = Bank {
                _id: id,
                role: "user".to_string(),
                email: email.clone(),
                password: digest(format!("{}@{}", username, phone_number)),
                phone_number,
                balance: money,
                qr_base64: generate_qr_base64(id).expect("unable to add")
            };
            app_state.bank.insert_one(bank_account)
                .await
                .map_err(| err | ApiResponse::text(500, err.to_string()))?;
            Ok(ApiResponse::text(201, "User registered successfully".to_string()))
        } 
        Err(err) => return Err(ApiResponse::text(500, format!("Database error: {}", err))),
    }
}


pub async fn send_email(
    to_email: &str,
    image_path: &str, 
) -> Result<(), lettre::transport::smtp::Error> {
    let smtp_server: &str = "smtp.hostinger.com";
    let smtp_username: &str = "brightwings@ranglerajasthan.in";
    let smtp_password: &str = "BrightWings@9783";
    let from_email: &str = "brightwings@ranglerajasthan.in";

    let subject = "🎟️ Your Pass to Rang Le Rajasthan – Let the Colors Begin!";

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
                font-size: 28px;
                text-align: center;
            }}
            h2 {{
                color: #2c3e50;
                font-size: 22px;
                margin-top: 20px;
            }}
            p {{
                font-size: 16px;
                margin: 10px 0;
            }}
            .highlight {{
                font-weight: bold;
                color: #e74c3c;
            }}
            .event-details {{
                background: #f8f9fa;
                padding: 15px;
                border-radius: 8px;
                margin-top: 10px;
            }}
        </style>
    </head>
    <body>
        <h1>🎟️ Your Pass to Rang Le Rajasthan – Let the Colors Begin!</h1>

        <p>Dear Customer,</p>

        <p>We’re thrilled to welcome you to <strong>Rang Le Rajasthan</strong>, Jaipur’s wildest fusion of <span class="highlight">colors, beats, and tomato mayhem! 🌈🍅</span> 
        Your ticket is ready, and the countdown to the madness has officially begun.</p>

        <h2>📩 Your E-Ticket is Attached!</h2>
        <p>Please find your digital ticket attached to this email. Ensure it’s saved on your phone or printed for seamless entry. 
        Lost it? Check spam or reach out to us ASAP.</p>

        <h2>📍 Event Details:</h2>
        <div class="event-details">
            <p><strong>Date:</strong> <span class="highlight">14 March 2025</span></p>
            <p><strong>Time:</strong> <span class="highlight">10:00 AM Onwards</span></p>
            <p><strong>Venue:</strong> <span class="highlight">Celebration Paradise, Sirsi Road, Jaipur</span></p>
            <p><strong>Lineup:</strong> DJ Kimmi Dubai, DJ Mehak Smoker, Marina Belly Dancer, RJ Gitanjali & more!</p>
        </div>

        <h2>🎉 Let’s Make Memories!</h2>
        <p>Tag us in your pre-Holi hype with <strong>#brightwingstours</strong> – we’ll feature the best posts!</p>

        <p>Can’t wait to see you there – let’s paint Jaipur in every shade of FUN! 🎨✨</p>

        <h2>Warm Regards,</h2>
        <p><strong>Bright Wings Travel & Tourism</strong></p>
        <p><a href="https://www.brightwingstravel.com" target="_blank">www.brightwingstravel.com</a></p>
    </body>
    </html>
    "#
);


    let image_data = fs::read(image_path).expect("Failed to read image file");

    let email = Message::builder()
        .from(Mailbox::new(None, from_email.parse().unwrap()))
        .to(Mailbox::new(None, to_email.parse().unwrap()))
        .subject(subject)
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(body.to_string()),
                )
                .singlepart(
                    Attachment::new(image_path.to_string())
                        .body(image_data, "image/png".parse().unwrap()), // Adjust MIME type as needed
                ),
        )
        .unwrap();

    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
    let mailer = SmtpTransport::relay(smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)?;

    Ok(())
}


fn generate_ticket_image(qr_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = (*utils::constants::DIR_PATH).clone();
    let template_path: String = format!("{}/qr_temp/template.png",dir_path);

    const QR_X: u32 = 1425;
    const QR_Y: u32 = 125;
    const QR_SIZE: u32 = 260;
    let qr_img = image::open(qr_path)?.resize_exact(QR_SIZE, QR_SIZE, image::imageops::FilterType::Lanczos3);
    
    let mut ticket = image::open(template_path)?.to_rgba8();
    
    overlay_image(&mut ticket, &qr_img.to_rgba8(), QR_X, QR_Y);
    
    ticket.save_with_format(output_path, ImageFormat::Png)?;
    
    println!("Generated: {}", output_path);
    Ok(())
}

fn overlay_image(base: &mut RgbaImage, overlay: &RgbaImage, x: u32, y: u32) {
    for oy in 0..overlay.height() {
        for ox in 0..overlay.width() {
            let pixel = overlay.get_pixel(ox, oy);
            base.put_pixel(x + ox, y + oy, *pixel);
        }
    }
}



async fn generate_qr_image(data: ObjectId, src: String) -> Result<(), ApiResponse> {

    let code = QrCode::new(data.to_string())
        .map_err(|err| ApiResponse::text(500, err.to_string()))?;
    
    let image = code.render::<Luma<u8>>().build();
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageOutputFormat::Png)
        .map_err(|err| ApiResponse::text(500, err.to_string()))?;
    let _ = image.save(src);
    Ok(())
}

fn generate_qr_base64(ticket_uuid: ObjectId) -> Result<String, String> {
    let code = QrCode::new(ticket_uuid.to_string())
        .map_err(|_| "Failed to generate QR code".to_owned())?;

    let image = code.render::<Luma<u8>>().build();

    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageOutputFormat::Png)
        .map_err(|_| "Failed to write image".to_owned())?;

    let base64_qr = STANDARD.encode(buffer.get_ref());
    let qr_code_url = format!("data:image/png;base64,{}", base64_qr);

    Ok(qr_code_url)
}

fn generate_ticket(ticket: TicketRequest) -> Ticket {
    Ticket {
        id: ObjectId::new(),
        username: ticket.username,
        email: ticket.email,
        phone_number: ticket.phone_number,
        ref_code: ticket.ref_code,
        category: ticket.category,
        additional_humans: ticket.additional_humans,
        children: ticket.children,
        used: false,
        amount: ticket.amount,
        is_online: true
    }
}


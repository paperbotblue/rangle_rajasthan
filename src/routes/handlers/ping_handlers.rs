use actix_web::{get, HttpResponse, Responder};

#[get("")]
async fn ping(
) -> impl Responder {
    HttpResponse::Ok() // 200 OK with no body
        .finish()
}



use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, HttpMessage, http::header::AUTHORIZATION ,Error, middleware::Next};
use crate::utils::{api_responses::ApiResponse, jwt::decode_jwt};


pub async fn check_auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth_header = req.headers().get(AUTHORIZATION);

    if auth_header.is_none() {
        return Err(Error::from(ApiResponse::text(401, "Unauthorized".to_string())));
    }

    let auth_header = auth_header.unwrap().to_str().map_err(|_| {
        Error::from(ApiResponse::text(401, "Invalid Authorization header".to_string()))
    })?;

    if !auth_header.starts_with("Bearer ") {
        return Err(Error::from(ApiResponse::text(401, "Invalid token format".to_string())));
    }

    let token = auth_header[7..].to_string();

    let claim = decode_jwt(token).map_err(|err| {
        Error::from(ApiResponse::text(401, format!("Invalid token: {}", err)))
    })?;

    req.extensions_mut().insert(claim.claims);

    next.call(req).await.map_err(|err| {
        Error::from(ApiResponse::text(500, format!("Internal server error: {}", err)))
    })
}

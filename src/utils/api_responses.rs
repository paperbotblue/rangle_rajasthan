use actix_web::{
    body::BoxBody, http::StatusCode, web, HttpResponse, Responder, ResponseError, HttpRequest,
    HttpResponseBuilder,
};
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Serialize)]
pub enum ApiResponseBody {
    Json(serde_json::Value),
    Image(Vec<u8>),
    Text(String),
}

#[derive(Debug)]
pub struct ApiResponse {
    pub status_code: u16,
    pub body: ApiResponseBody,
    response_code: StatusCode,
    pub content_type: String,
}

impl ApiResponse {
    pub fn json(status_code: u16, json_body: serde_json::Value) -> Self {
        ApiResponse {
            status_code,
            body: ApiResponseBody::Json(json_body),
            response_code: StatusCode::from_u16(status_code).unwrap(),
            content_type: "application/json".to_string(),
        }
    }

    pub fn text(status_code: u16, text: String) -> Self {
        ApiResponse {
            status_code,
            body: ApiResponseBody::Text(text),
            response_code: StatusCode::from_u16(status_code).unwrap(),
            content_type: "text/plain".to_string(),
        }
    }

    pub fn image(status_code: u16, image_bytes: Vec<u8>, content_type: &str) -> Self {
        ApiResponse {
            status_code,
            body: ApiResponseBody::Image(image_bytes),
            response_code: StatusCode::from_u16(status_code).unwrap(),
            content_type: content_type.to_string(),
        }
    }
}

impl Responder for ApiResponse {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut response = HttpResponseBuilder::new(self.response_code);
        response.insert_header(("Content-Type", self.content_type.clone()));

        match self.body {
            ApiResponseBody::Json(json_body) => response.body(serde_json::to_string(&json_body).unwrap()),
            ApiResponseBody::Text(text) => response.body(text),
            ApiResponseBody::Image(image_bytes) => response.body(image_bytes),
        }
    }
}

impl Display for ApiResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Response: {:?} \n Status Code: {}", self.body, self.status_code)
    }
}

impl ResponseError for ApiResponse {
    fn status_code(&self) -> StatusCode {
        self.response_code
    }
}


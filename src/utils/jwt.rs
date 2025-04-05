use std::{
    future, env
};
use actix_web::{self, HttpMessage, FromRequest};
use chrono::{Utc, Duration};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::utils::constants;

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub email: String,
    pub _id: ObjectId
}

impl FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = future::Ready<Result<Self, Self::Error>>;
    
    fn from_request(
        req: &actix_web::HttpRequest, 
        payload: &mut actix_web::dev::Payload
    ) -> std::future::Ready<Result<Claims, actix_web::Error>> {
        match req.extensions_mut().get::<Claims>() {
            Some(claims) => future::ready(Ok(claims.clone())),
            None => future::ready(Err(actix_web::error::ErrorBadRequest("Bad Claims")))
        }
    }
}

pub fn encode_jwt(email: String, _id: ObjectId) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expire = Duration::hours(24);

    let claims = Claims {
        exp: (now + expire).timestamp() as usize,
        iat: now.timestamp() as usize,
        _id,
        email
    };
    let secret = (*constants::SECRET).clone();

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn decode_jwt(token: String) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = (*constants::SECRET).clone();


    let claim_data: Result<TokenData<Claims>, jsonwebtoken::errors::Error> = decode(
        &token, 
        &DecodingKey::from_secret(secret.as_ref()), 
        &Validation::default()
    );

    claim_data
}

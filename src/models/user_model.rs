
use mongodb::bson::{
    self, oid::ObjectId
};
use serde::{
    Serialize, Deserialize
};

use crate::routes::handlers::auth_handlers::register;

#[derive(Debug, Serialize, Deserialize)]
pub struct User{
    pub _id: ObjectId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub verified: bool,
    pub role: String,
    pub created_at: bson::DateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegister{
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin{
    pub email: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdate{
    pub email: String,
    pub password: String,
    pub verified: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDelete{
    pub email: String,
}

impl TryFrom<UserRegister> for User{
    type Error = Box<dyn std::error::Error>;

    fn try_from(item: UserRegister) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            username: item.username,
            email: item.email,
            password: item.password,
            verified: false,
            role: "superadmin".to_string(),
            created_at: bson::DateTime::now()
        })
    }
}

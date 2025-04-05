use mongodb::{
    bson::{doc, oid::ObjectId}, 
    error::Error, results::{DeleteResult, InsertOneResult}, Collection,
};
use mongodb::bson::Uuid as BsonUuid;
use sha256::digest;
use crate::models::{referal_code::ReferalCode,ticket_model::Ticket, user_model::{User,UserLogin}, bank_models::{Bank, Otp,RechargeOffline, BankTransactions, Recharge}};


pub struct Database {
    pub referral: Collection<ReferalCode>,
    pub ticket: Collection<Ticket>,
    pub admin: Collection<User>,
    pub bank: Collection<Bank>,
    pub bank_transactions: Collection<BankTransactions>,
    pub recharge: Collection<Recharge>,
    pub recharge_offline: Collection<RechargeOffline>,
    pub otp: Collection<Otp>
}

impl Database {
    pub async fn create_admin(&self, admin: User) -> Result<InsertOneResult, Error> {
        let hashed_password = digest(&admin.password);
        let admin_temp = User{
            _id: admin._id,
            username: admin.username,
            email: admin.email,
            password: hashed_password, 
            verified: admin.verified,
            role: admin.role,
            created_at: admin.created_at
        };

        let result = self
            .admin
            .insert_one(admin_temp)
            .await
            .ok()
            .expect("error creating admin");

        Ok(result)
    }
    pub async fn verify_admin(&self, login: UserLogin) -> Result< User, mongodb::error::Error> {
        let filter = doc! {
            "email": &login.email
        };

        match self.admin.find_one(filter).await? {
            Some(admin) => Ok(admin),
            None => Err(mongodb::error::Error::custom("Invalid credentials")), // Handle missing user
        }
    }


    pub async fn create_ticket(&self, ticket: &Ticket) -> Result<InsertOneResult, Error> {
        let result = self
            .ticket
            .insert_one(ticket)
            .await
            .ok()
            .expect("Error creating ticket");

        Ok(result)
    }

    pub async fn find_ticket(&self, ticket_uuid: ObjectId) -> Result<Option<Ticket>, Error> {
        let filter = doc! {
            "_id": ticket_uuid
        };

        match self.ticket.find_one(filter).await {
            Ok(Some(ticket)) => Ok(Some(ticket)),
            Ok(None) => Ok(None),              
            Err(e) => Err(e),     
        }
    }
    pub async fn delete_ticket(&self, ticket_uuid: ObjectId) -> Result<DeleteResult, Error> {
        let result = self
            .ticket
            .delete_one(doc! {
                "_id": ticket_uuid
            })
            .await
            .ok()
            .expect("Error deleting ticket");

        Ok(result)
    }
}


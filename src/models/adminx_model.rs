// adminx/src/models/adminx_model.rs
use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId, DateTime as BsonDateTime};
use chrono::Utc;
use bcrypt::verify;

use crate::{
    utils::{
        auth::{
            get_adminx_database
        },
        auth::{
            AdminxStatus
        },
    }
};



#[derive(Debug, Serialize, Deserialize)]
pub struct AdminxUser {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub password: String, // hashed
    pub delete: bool,
    pub status: AdminxStatus,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}


impl AdminxUser {
    pub fn verify_password(&self, plain: &str) -> bool {
        verify(plain, &self.password).unwrap_or(false)
    }
}


pub async fn get_admin_by_email(email: &str) -> Option<AdminUser> {
    let db = get_adminx_database();
    let collection = db.collection::<AdminxUser>("adminxs");
    collection.find_one(doc! { "email": email }, None).await.ok().flatten()
}
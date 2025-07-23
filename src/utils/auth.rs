// adminx/src/utils/auth.rs
use crate::models::adminx_model::{AdminxUser};
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
};
use bcrypt::{hash, DEFAULT_COST};
use anyhow::{Result, Context};
use crate::{custom_error_nonexpression, custom_error_expression};
use serde::{Serialize, Deserialize};
use crate::{
    utils::{
        database::{
            get_adminx_database
        },
        ubson::{
            convert_to_bson
        },
    }
};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AdminxStatus {
    Active,
    Inactive,
}


pub enum InitOutcome {
    Created,
    Updated,
}




#[derive(Debug)]
pub struct NewAdminxUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub status: AdminxStatus,
    pub delete: bool,
}

pub async fn initiate_auth(adminx: NewAdminxUser) -> Result<InitOutcome, actix_web::Error> {
    let db = get_adminx_database();
    let collection = db.collection::<AdminxUser>("adminxs");

    
    let now = BsonDateTime::now();
    let hashed_pwd = hash(&adminx.password, DEFAULT_COST)
    	.map_err(|e| custom_error_expression!(bad_request, 400, format!("Failed to hash password: {e}")))?;


    match collection.find_one(doc! { "email": &adminx.email }, None).await {
        Ok(Some(_exist)) => {
        	let status_bson = convert_to_bson(&adminx.status)?;
            let update_doc = doc! {
                "username": adminx.username,
                "password": hashed_pwd,
                "delete": adminx.delete,
                "status": status_bson,
                "updated_at": now,
            };

            collection.update_one(
                doc! { "email": &adminx.email },
                doc! { "$set": update_doc },
                None,
            )
            .await
            .map_err(|e| custom_error_expression!(bad_request, 400, e.to_string()))?;
            Ok(InitOutcome::Updated)
        }
        Ok(None) => {
            let new_user = AdminxUser {
                id: None,
                username: adminx.username,
                email: adminx.email,
                password: hashed_pwd,
                delete: adminx.delete,
                status: adminx.status,
                created_at: now,
                updated_at: now,
            };

            collection.insert_one(new_user, None)
                .await
                .map_err(|e| custom_error_expression!(invalid_request, 422, format!("User creation failed: {e}")))?;
            Ok(InitOutcome::Created)
        }
        Err(e) => {
            return Err(custom_error_nonexpression!(internal_error, 500, format!("DB error: {e}")));
        }
    }
}

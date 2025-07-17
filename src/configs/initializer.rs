// src/configs/initializer.rs
use crate::helpers::template_helper::ADMINX_TEMPLATES;
use log::{info};
use mongodb::Database;
use anyhow::{anyhow, Error as AnyhowError};
use crate::utils::{
    database::{
        initiate_database
    },
};

pub async fn adminx_initialize(db: Database) -> Result<(), AnyhowError> {
	let initiate_database(db);
	// let _ = ADMINX_TEMPLATES.len();
	info!("AdminX initialized successfully");
	Ok(())
}
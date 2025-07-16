// src/configs/initializer.rs
use crate::helpers::template_helper::ADMINX_TEMPLATES;
use log::{info};
use anyhow::{anyhow, Error as AnyhowError};

pub async fn adminx_initialize() -> Result<(), AnyhowError> {
	// let _ = ADMINX_TEMPLATES.len();
	info!("AdminX initialized successfully");
	Ok(())
}
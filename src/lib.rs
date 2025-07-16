

pub mod resource;
pub mod filters;
pub mod pagination;
pub mod error;
pub mod router;
pub mod menu;
pub mod registry;
pub mod health;
pub mod middleware;
pub mod nested;
pub mod utils;
pub mod actions;
pub mod helpers;
pub mod controllers;
pub mod configs;

pub mod schemas;

pub use schemas::adminx_schema::AdminxSchema;
pub use resource::AdmixResource;
pub use configs::initializer::adminx_initialize;
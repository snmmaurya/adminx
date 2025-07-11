use actix_web::{web, HttpRequest, HttpResponse, Scope};
use serde_json::Value;
use std::sync::Arc;
use crate::resource::AdmixResource;

/// Dynamically mount all CRUD routes for a resource
pub fn register_handlers(
    resource_name: &'static str,
    resource: Box<dyn AdmixResource>,
) -> Scope {
    let resource = Arc::new(resource); // Clone-safe for handlers

    web::scope(&format!("/{}", resource_name))
        .route("", web::get().to({
            let resource = Arc::clone(&resource);
            move |req: HttpRequest| {
                let query_string = req.query_string().to_string();
                let resource = Arc::clone(&resource);
                async move { resource.list(&req, query_string).await }
            }
        }))
        .route("", web::post().to({
            let resource = Arc::clone(&resource);
            move |req: HttpRequest, payload: web::Json<Value>| {
                let resource = Arc::clone(&resource);
                async move { resource.create(&req, payload.into_inner()).await }
            }
        }))
        .route("/{id}", web::get().to({
            let resource = Arc::clone(&resource);
            move |req: HttpRequest, id: web::Path<String>| {
                let resource = Arc::clone(&resource);
                async move { resource.get(&req, id.into_inner()).await }
            }
        }))
        .route("/{id}", web::put().to({
            let resource = Arc::clone(&resource);
            move |req: HttpRequest, id: web::Path<String>, payload: web::Json<Value>| {
                let resource = Arc::clone(&resource);
                async move {
                    resource.update(&req, id.into_inner(), payload.into_inner()).await
                }
            }
        }))
        .route("/{id}", web::delete().to({
            let resource = Arc::clone(&resource);
            move |req: HttpRequest, id: web::Path<String>| {
                let resource = Arc::clone(&resource);
                async move { resource.delete(&req, id.into_inner()).await }
            }
        }))
}

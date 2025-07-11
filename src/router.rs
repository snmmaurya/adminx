// src/router.rs


use actix_web::{web, Scope, HttpRequest, HttpResponse, HttpMessage};
use serde_json::Value;

use crate::middleware::role_guard::RoleGuard;
use crate::nested::AdmixNestedResource;
use crate::registry::all_resources;
use crate::resource::AdmixResource;
use crate::actions::CustomAction;
use crate::menu::MenuAction;
use crate::utils::rbac::has_permission;

fn extract_roles_from_request(req: &HttpRequest) -> Vec<String> {
    // Replace this stub with actual logic to extract roles from request/session/token
    req.extensions()
        .get::<Vec<String>>()
        .cloned()
        .unwrap_or_default()
}

pub fn register_all_admix_routes() -> Scope {
    let mut scope = web::scope("/adminx");

    for resource in all_resources() {
        let base_path = resource.base_path().to_string();
        let allowed_roles = resource.allowed_roles();
        let service = register_admix_resource_routes(resource);
        scope = scope.service(
            web::scope(&base_path)
                .wrap(RoleGuard { allowed_roles })
                .service(service),
        );
    }

    scope
}

pub fn register_admix_resource_routes(resource: Box<dyn AdmixResource>) -> Scope {
    let base_path = resource.base_path().to_string();
    let mut scope = web::scope("");

    // GET / => list
    let list_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::get().to(move |req: HttpRequest, query: web::Query<String>| {
            let resource = list_resource.clone_box();
            let roles = extract_roles_from_request(&req);
            async move {
                if has_permission(resource.as_ref(), &roles, MenuAction::List) {
                    resource.list(&req, query.into_inner()).await
                } else {
                    HttpResponse::Forbidden().body("Not allowed to list")
                }
            }
        }),
    );

    // POST / => create
    let create_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::post().to(move |req: HttpRequest, body: web::Json<Value>| {
            let resource = create_resource.clone_box();
            let roles = extract_roles_from_request(&req);
            async move {
                if has_permission(resource.as_ref(), &roles, MenuAction::Create) {
                    resource.create(&req, body.into_inner()).await
                } else {
                    HttpResponse::Forbidden().body("Not allowed to create")
                }
            }
        }),
    );

    // GET /{id} => get
    let get_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::get().to(move |req: HttpRequest, path: web::Path<String>| {
            let resource = get_resource.clone_box();
            let roles = extract_roles_from_request(&req);
            async move {
                if has_permission(resource.as_ref(), &roles, MenuAction::View) {
                    resource.get(&req, path.into_inner()).await
                } else {
                    HttpResponse::Forbidden().body("Not allowed to view")
                }
            }
        }),
    );

    // PUT /{id} => update
    let update_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::put().to(move |req: HttpRequest, path: web::Path<String>, body: web::Json<Value>| {
            let resource = update_resource.clone_box();
            let roles = extract_roles_from_request(&req);
            async move {
                if has_permission(resource.as_ref(), &roles, MenuAction::Edit) {
                    resource.update(&req, path.into_inner(), body.into_inner()).await
                } else {
                    HttpResponse::Forbidden().body("Not allowed to update")
                }
            }
        }),
    );

    // DELETE /{id} => delete
    let delete_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::delete().to(move |req: HttpRequest, path: web::Path<String>| {
            let resource = delete_resource.clone_box();
            let roles = extract_roles_from_request(&req);
            async move {
                if has_permission(resource.as_ref(), &roles, MenuAction::Delete) {
                    resource.delete(&req, path.into_inner()).await
                } else {
                    HttpResponse::Forbidden().body("Not allowed to delete")
                }
            }
        }),
    );

    // Nested resources
    for nested in resource.nested_resources() {
        scope = scope.service(nested.as_scope());
    }

    // Custom dynamic actions
    for action in resource.custom_actions() {
        let path = format!("/{{id}}/{}", action.name);
        match action.method.as_ref() {
            "POST" => {
                scope = scope.route(&path, web::post().to(action.handler));
            }
            "GET" => {
                scope = scope.route(&path, web::get().to(action.handler));
            }
            _ => println!("Unsupported method: {}", action.method),
        }
    }

    web::scope(&base_path).service(scope)
}

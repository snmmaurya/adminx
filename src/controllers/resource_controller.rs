// adminx/src/controllers/resource_controller.rs
use actix_web::{web, HttpRequest, HttpResponse, Scope};
use serde_json::Value;
use std::sync::Arc;
use tera::Context;

use crate::AdmixResource;
use crate::helpers::form_helper::extract_fields_for_form;
use crate::helpers::template_helper::render_template;
// use crate::helpers::permission_helper::{extract_roles_from_request, has_permission};
// use crate::models::MenuAction;


/// Register all UI + API routes for a resource
pub fn register_admix_resource_routes(resource: Box<dyn AdmixResource>) -> Scope {
    let base_path = resource.base_path().to_string();
    let mut scope = web::scope("");

    let list_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::get().to(move |req: HttpRequest, query: web::Query<String>| {
            let resource = list_resource.clone_box();
            // let roles = extract_roles_from_request(&req);
            async move {
                // if has_permission(resource.as_ref(), &roles, MenuAction::List) {
                    resource.list(&req, query.into_inner()).await
                // } else {
                //     HttpResponse::Forbidden().body("Not allowed to list")
                // }
            }
        }),
    );

    let create_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::post().to(move |req: HttpRequest, body: web::Json<Value>| {
            let resource = create_resource.clone_box();
            // let roles = extract_roles_from_request(&req);
            async move {
                // if has_permission(resource.as_ref(), &roles, MenuAction::Create) {
                    resource.create(&req, body.into_inner()).await
                // } else {
                //     HttpResponse::Forbidden().body("Not allowed to create")
                // }
            }
        }),
    );

    let get_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::get().to(move |req: HttpRequest, path: web::Path<String>| {
            let resource = get_resource.clone_box();
            // let roles = extract_roles_from_request(&req);
            async move {
                // if has_permission(resource.as_ref(), &roles, MenuAction::View) {
                    resource.get(&req, path.into_inner()).await
                // } else {
                //     HttpResponse::Forbidden().body("Not allowed to view")
                // }
            }
        }),
    );

    let update_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::put().to(move |req: HttpRequest, path: web::Path<String>, body: web::Json<Value>| {
            let resource = update_resource.clone_box();
            // let roles = extract_roles_from_request(&req);
            async move {
                // if has_permission(resource.as_ref(), &roles, MenuAction::Edit) {
                    resource.update(&req, path.into_inner(), body.into_inner()).await
                // } else {
                //     HttpResponse::Forbidden().body("Not allowed to update")
                // }
            }
        }),
    );

    let delete_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::delete().to(move |req: HttpRequest, path: web::Path<String>| {
            let resource = delete_resource.clone_box();
            // let roles = extract_roles_from_request(&req);
            async move {
                // if has_permission(resource.as_ref(), &roles, MenuAction::Delete) {
                    resource.delete(&req, path.into_inner()).await
                // } else {
                //     HttpResponse::Forbidden().body("Not allowed to delete")
                // }
            }
        }),
    );

    // Extended UI Routes
    let resource_arc = Arc::new(resource.clone_box());
    let resource_name = resource_arc.base_path().to_string();

    scope = scope
        .route("/list", web::get().to({
            let resource = Arc::clone(&resource_arc);
            move |req: HttpRequest| {
                let query_string = req.query_string().to_string();
                let resource = Arc::clone(&resource);
                async move { resource.list(&req, query_string).await }
            }
        }))
        .route("/view/{id}", web::get().to({
            let resource = Arc::clone(&resource_arc);
            move |req: HttpRequest, id: web::Path<String>| {
                let resource = Arc::clone(&resource);
                async move { resource.get(&req, id.into_inner()).await }
            }
        }))
        .route("/new", web::get().to({
        let resource = Arc::clone(&resource_arc);
        let resource_name = resource_arc.base_path().to_string();
        move |req: HttpRequest, tmpl: web::Data<tera::Tera>| {
            let resource = Arc::clone(&resource);
            let resource_name = resource_name.clone(); // ✅ clone instead of move
            async move {
                println!("✅ /new route hit for {}", resource.base_path());
                let form = resource.form_structure().unwrap_or_default();

                let mut ctx = Context::new();
                ctx.insert("resource_name", &resource_name);
                ctx.insert("base_path", &format!("/adminx/{}", resource_name));
                ctx.insert("fields", &extract_fields_for_form(&form));
                ctx.insert("menus", &resource.build_adminx_menus());

                render_template("new.html.tera", ctx).await
            }
        }
    }));

    // Nested resources
    for nested in resource_arc.nested_resources() {
        scope = scope.service(nested.as_scope());
    }

    // Custom actions
    for action in resource_arc.custom_actions() {
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

    scope
}

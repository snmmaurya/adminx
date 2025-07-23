// adminx/src/router.rs

use actix_web::{web, Scope, HttpRequest, HttpResponse, HttpMessage};
use serde_json::Value;

use crate::middleware::role_guard::RoleGuard;
use crate::nested::AdmixNestedResource;
use crate::registry::all_resources;
use crate::resource::AdmixResource;
use crate::actions::CustomAction;
use crate::menu::MenuAction;
use crate::utils::rbac::has_permission;
use crate::controllers::{
    dashboard_controller::{
        adminx_home
    },
    resource_controller::{
        register_admix_resource_routes
    }
};
use crate::controllers::auth_controller::{login_form, login_action};


fn extract_roles_from_request(req: &HttpRequest) -> Vec<String> {
    // Replace this stub with actual logic to extract roles from request/session/token
    req.extensions()
        .get::<Vec<String>>()
        .cloned()
        .unwrap_or_default()
}

pub fn register_all_admix_routes() -> Scope {
    let mut scope = web::scope("/adminx")
        .route("/login", web::get().to(login_form))
        .route("/login", web::post().to(login_action));

    for resource in all_resources() {
        let allowed_roles = resource.allowed_roles();
        let resource_scope = register_admix_resource_routes(resource)
            .wrap(RoleGuard { allowed_roles });

        scope = scope.service(resource_scope);
    }

    scope
}

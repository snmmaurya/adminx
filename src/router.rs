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


fn extract_roles_from_request(req: &HttpRequest) -> Vec<String> {
    // Replace this stub with actual logic to extract roles from request/session/token
    req.extensions()
        .get::<Vec<String>>()
        .cloned()
        .unwrap_or_default()
}

pub fn register_all_admix_routes() -> Scope {
    let mut scope = web::scope("/adminx")
        .route("", web::get().to(adminx_home)); // handles /adminx

    for resource in all_resources() {
        let allowed_roles = resource.allowed_roles();
        let service = register_admix_resource_routes(resource);
        // Don't re-scope with base_path; it's already done inside `register_admix_resource_routes`
        scope = scope.service(service.wrap(RoleGuard { allowed_roles }));

        // scope = scope.service();
    }

    scope
}


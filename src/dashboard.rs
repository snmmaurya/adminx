use actix_web::{web, Scope};
use std::collections::HashMap;
use crate::handlers::register_handlers;
use serde::{Serialize, Deserialize};
use crate::resource::{AdmixResource};
use crate::menu::{MenuItem, MenuAction};
use std::sync::RwLock;
use once_cell::sync::Lazy;


static RESOURCE_REGISTRY: Lazy<RwLock<Vec<Box<dyn AdmixResource>>>> = Lazy::new(|| RwLock::new(Vec::new()));


/// Register an AdmixResource globally (e.g., from `main.rs`)
pub fn register_resource(resource: Box<dyn AdmixResource>) {
    RESOURCE_REGISTRY.write().unwrap().push(resource);
}

/// Collect all the menus from registered resources
pub fn get_registered_menus() -> Vec<MenuItem> {
    RESOURCE_REGISTRY
        .read()
        .unwrap()
        .iter()
        .filter_map(|r| r.generate_menu())
        .collect()
}



//#[derive(Clone, Serialize, Deserialize)]
pub struct AdmixDashboard {
    resources: HashMap<&'static str, Box<dyn AdmixResource>>,
}

impl AdmixDashboard {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn register<R: AdmixResource + 'static>(mut self) -> Self {
        let resource_instance = R::new();
        let resource = resource_instance.resource_name();
        self.resources.insert(resource, Box::new(resource_instance));
        self
    }

    pub fn into_scope(self) -> Scope {
        let mut scope = web::scope("/admin");

        for (name, resource) in self.resources {
            scope = scope.service(register_handlers(name, resource));
        }

        scope
    }


    pub fn clone_into_scope(&self) -> actix_web::Scope {
        let mut cloned = AdmixDashboard::new();

        for (name, resource) in &self.resources {
            cloned.resources.insert(name, resource.clone_box()); // ✅ now works
        }

        cloned.into_scope()
    }
}




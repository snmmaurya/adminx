// ===============================
// src/registry.rs
// ===============================
use crate::resource::AdmixResource;
use std::sync::RwLock;
use lazy_static::lazy_static;

lazy_static! {
    static ref RESOURCE_REGISTRY: RwLock<Vec<Box<dyn AdmixResource>>> = RwLock::new(vec![]);
}

/// Register a resource globally
pub fn register_resource(resource: Box<dyn AdmixResource>) {
    RESOURCE_REGISTRY.write().unwrap().push(resource);
}

/// Get all registered resources (cloned)
pub fn all_resources() -> Vec<Box<dyn AdmixResource>> {
    RESOURCE_REGISTRY
        .read()
        .unwrap()
        .iter()
        .map(|r| r.clone_box())
        .collect()
}

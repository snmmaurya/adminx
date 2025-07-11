// src/route.rs

use actix_web::web::{self, ServiceConfig};
use mongodb::Database;

use crate::dashboard::AdmixDashboard;

pub fn configure_dashboard(cfg: &mut ServiceConfig, dashboard: &AdmixDashboard, db: Database) {
    for resource in dashboard.resources() {
        let scope = resource.routes(db.clone());
        cfg.service(scope);
    }
}

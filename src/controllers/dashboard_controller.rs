// src/controllers/dashboard_controller.rs

use actix_web::{get, HttpResponse, Responder};
use tera::Context;
use crate::registry::get_registered_menus;
use crate::helpers::template_helper::render_template;


pub async fn adminx_home() -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("menus", &get_registered_menus());
    render_template("layout.html.tera", ctx).await
}

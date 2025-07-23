// adminx/src/controllers/auth_controller.rs

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_session::{Session};
use serde::{Deserialize};
use tera::Context;
use crate::helpers::template_helper::render_template;
use crate::utils::jwt::{create_jwt_token}; // You'll implement this
use crate::models::adminx_model::{AdminxUser, get_admin_by_email}; // You'll define this
use crate::registry::get_registered_menus;


#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

/// GET /adminx/login - Show login page
pub async fn login_form() -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("menus", &get_registered_menus());
    render_template("login.html.tera", ctx).await
}

/// POST /adminx/login - Authenticate and issue JWT


pub async fn login_action(
    form: web::Form<LoginForm>,
    session: Session,
) -> impl Responder {
    let user = get_admin_by_email(&form.email).await;

    match user {
        Some(admin) => {
            if admin.verify_password(&form.password) {
                match create_jwt_token(
                    &admin.id.expect("Missing AdminxUser ID").to_string(),
                    &admin.email,
                    "admin"
                ) {
                    Ok(token) => {
                        // ✅ Store token in session
                        if let Err(err) = session.insert("admintoken", token) {
                            return HttpResponse::InternalServerError().body(format!("Session error: {err}"));
                        }

                        HttpResponse::Found()
                            .append_header(("Location", "/adminx"))
                            .finish()
                    }
                    Err(_) => HttpResponse::InternalServerError().body("Token generation failed"),
                }
            } else {
                HttpResponse::Unauthorized().body("Invalid password")
            }
        }
        None => HttpResponse::Unauthorized().body("Admin not found"),
    }
}

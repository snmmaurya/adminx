// adminx/src/controllers/auth_controller.rs

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize};
use tera::Context;
use crate::helpers::template_helper::render_template;
use crate::utils::jwt::{create_jwt_token}; // You'll implement this
use crate::models::adminx_model::{AdminxUser, get_admin_by_email}; // You'll define this


#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

/// GET /adminx/login - Show login page
pub async fn login_form(tmpl: web::Data<tera::Tera>) -> impl Responder {
    let ctx = Context::new();
    render_template("login.html.tera", ctx).await
}

/// POST /adminx/login - Authenticate and issue JWT
pub async fn login_action(
    form: web::Form<LoginForm>,
) -> impl Responder {
    let user = get_admin_by_email(&form.email).await;

    match user {
        Some(admin) => {
            if admin.verify_password(&form.password) {
                match create_jwt_token(&admin.id.to_string(), Some("admin")) {
                    Ok(token) => {
                        HttpResponse::Found()
                            .append_header(("Location", "/adminx"))
                            .cookie(
                                actix_web::cookie::Cookie::build("admintoken", token)
                                    .path("/")
                                    .http_only(true)
                                    .finish()
                            )
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

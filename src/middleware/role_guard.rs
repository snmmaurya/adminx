
// adminx/src/middleware/role_guard.rs

use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, forward_ready}, Error, HttpMessage};
use futures_util::future::{LocalBoxFuture, ready};
use std::rc::Rc;
use std::task::{Context, Poll};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    roles: Vec<String>, // roles from token
}

pub struct RoleGuard {
    pub allowed_roles: Vec<String>,
}

impl<S, B> Transform<S, ServiceRequest> for RoleGuard
where
    S: actix_service::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RoleGuardMiddleware<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let allowed_roles = self.allowed_roles.clone();
        Box::pin(async move {
            Ok(RoleGuardMiddleware {
                service: Rc::new(service),
                allowed_roles,
            })
        })
    }
}

pub struct RoleGuardMiddleware<S> {
    service: Rc<S>,
    allowed_roles: Vec<String>,
}

impl<S, B> actix_service::Service<ServiceRequest> for RoleGuardMiddleware<S>
where
    S: actix_service::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);
        let allowed = self.allowed_roles.clone();

        Box::pin(async move {
            // Extract token from `Authorization` header
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(token_str) = auth_header.to_str() {
                    let token = token_str.strip_prefix("Bearer ").unwrap_or(token_str);

                    // Decode JWT
                    if let Ok(token_data) = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret("secret".as_bytes()), // use env in prod
                        &Validation::default(),
                    ) {
                        let user_roles: HashSet<String> = token_data.claims.roles.into_iter().collect();

                        for role in allowed {
                            if user_roles.contains(&role) {
                                return svc.call(req).await;
                            }
                        }
                    }
                }
            }

            Err(actix_web::error::ErrorUnauthorized("Access denied"))
        })
    }
}


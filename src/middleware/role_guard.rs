
// adminx/src/middleware/role_guard.rs

// adminx/src/middleware/role_guard.rs

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::{
    collections::HashSet,
    env,
    future::Ready,
    rc::Rc,
    task::{Context, Poll},
};

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    email: String,
    role: String,
    roles: Vec<String>,
}

pub struct RoleGuard {
    pub allowed_roles: Vec<String>,
}

impl<S, B> Transform<S, ServiceRequest> for RoleGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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

impl<S, B> Service<ServiceRequest> for RoleGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);
        let allowed_roles = self.allowed_roles.clone();

        println!("allowed_roles: {:?}", allowed_roles.clone());
        println!("svc: {:?}", svc.clone());

        Box::pin(async move {
            let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
                actix_web::error::ErrorInternalServerError("JWT_SECRET not set in env")
            })?;

            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(token_str) = auth_header.to_str() {
                    let token = token_str.strip_prefix("Bearer ").unwrap_or(token_str);

                    if let Ok(token_data) = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(jwt_secret.as_bytes()),
                        &Validation::default(),
                    ) {
                        let claims = token_data.claims;

                        let user_roles: HashSet<String> = {
                            let mut roles = claims.roles.clone();
                            roles.push(claims.role.clone()); // include `role` if separate
                            roles.into_iter().collect()
                        };

                        if allowed_roles.iter().any(|role| user_roles.contains(role)) {
                            // Optionally store claims in request extensions
                            req.extensions_mut().insert(claims);
                            return svc.call(req).await;
                        }
                    }
                }
            }

            Err(actix_web::error::ErrorUnauthorized("Access denied"))
        })
    }
}

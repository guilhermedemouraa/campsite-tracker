// src/auth/middleware.rs
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, Result,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use uuid::Uuid;

use super::jwt::JwtService;

/// Middleware for handling authentication by verifying JWT tokens
/// and extracting user information from the request.
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            jwt_service: JwtService::new(),
        }))
    }
}

/// Service that implements the authentication middleware logic
pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_service: JwtService,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let jwt_service = self.jwt_service.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| {
                    if h.starts_with("Bearer ") {
                        Some(h.strip_prefix("Bearer ").unwrap())
                    } else {
                        None
                    }
                });

            let token = match auth_header {
                Some(token) => token,
                None => {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "missing_token",
                        "message": "Authorization token is required"
                    }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // Verify token and extract user ID
            let user_id = match jwt_service.extract_user_id_from_token(token) {
                Ok(user_id) => user_id,
                Err(_) => {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "invalid_token",
                        "message": "Invalid or expired token"
                    }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // Add user ID to request extensions
            req.extensions_mut().insert(user_id);

            // Continue with the request
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

/// Helper function to extract user ID from request
pub fn extract_user_id(req: &ServiceRequest) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}

/// Custom extractor for authenticated user ID
pub struct AuthenticatedUser(pub Uuid);

impl actix_web::FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let user_id = req.extensions().get::<Uuid>().copied();

        ready(match user_id {
            Some(id) => Ok(AuthenticatedUser(id)),
            None => Err(actix_web::error::ErrorUnauthorized(
                "User not authenticated",
            )),
        })
    }
}

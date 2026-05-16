use std::{
    rc::Rc,
    task::{Context, Poll},
};

use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    web,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};

use crate::{
    errors::app_error::AppError,
    shared::{extractor::AuthenticatedUser, jwt::TokenType, state::AppState},
};

#[derive(Clone, Default)]
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
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
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

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let Some(state) = req.app_data::<web::Data<AppState>>() else {
                let response = req.into_response(
                    AppError::Internal("application state is missing".into())
                        .error_response()
                        .map_into_right_body(),
                );
                return Ok(response);
            };

            let token = match bearer_token(req.request()) {
                Ok(token) => token,
                Err(error) => {
                    let response = req.into_response(error.error_response().map_into_right_body());
                    return Ok(response);
                }
            };

            let claims = match state.jwt_service.decode_token(token) {
                Ok(claims) => claims,
                Err(error) => {
                    let response = req.into_response(error.error_response().map_into_right_body());
                    return Ok(response);
                }
            };

            if claims.token_type != TokenType::Access {
                let response = req.into_response(
                    AppError::Unauthorized("access token required".into())
                        .error_response()
                        .map_into_right_body(),
                );
                return Ok(response);
            }

            req.extensions_mut().insert(AuthenticatedUser {
                user_id: claims.sub,
                role: claims.role,
            });

            let response = service.call(req).await?.map_into_left_body();
            Ok(response)
        })
    }
}

fn bearer_token(request: &actix_web::HttpRequest) -> Result<&str, AppError> {
    let header_value = request
        .headers()
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;

    let value = header_value
        .to_str()
        .map_err(|_| AppError::Unauthorized("invalid Authorization header".into()))?;

    value
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("expected Bearer token".into()))
}

trait AppErrorHttpResponse {
    fn error_response(self) -> HttpResponse;
}

impl AppErrorHttpResponse for AppError {
    fn error_response(self) -> HttpResponse {
        <Self as actix_web::ResponseError>::error_response(&self)
    }
}

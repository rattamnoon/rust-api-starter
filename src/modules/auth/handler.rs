use actix_web::{HttpResponse, web};
use validator::Validate;

use crate::{
    errors::app_error::AppError,
    modules::auth::dto::{LoginRequest, RefreshTokenRequest, RegisterRequest},
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Register a new account", body = crate::modules::auth::dto::AuthResponse),
        (status = 400, description = "Validation error", body = ErrorResponseBody),
        (status = 409, description = "Email already exists", body = ErrorResponseBody)
    )
)]
pub async fn register(
    state: web::Data<AppState>,
    request: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state.auth_service.register(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login succeeded", body = crate::modules::auth::dto::AuthResponse),
        (status = 400, description = "Validation error", body = ErrorResponseBody),
        (status = 401, description = "Invalid credentials", body = ErrorResponseBody)
    )
)]
pub async fn login(
    state: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state.auth_service.login(request.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "Auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Rotate access and refresh tokens", body = crate::modules::auth::dto::AuthResponse),
        (status = 400, description = "Validation error", body = ErrorResponseBody),
        (status = 401, description = "Invalid refresh token", body = ErrorResponseBody)
    )
)]
pub async fn refresh(
    state: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state.auth_service.refresh(request.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    request_body = RefreshTokenRequest,
    responses(
        (status = 204, description = "Refresh token revoked"),
        (status = 400, description = "Validation error", body = ErrorResponseBody),
        (status = 401, description = "Unauthorized", body = ErrorResponseBody)
    )
)]
pub async fn logout(
    state: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
    _current_user: CurrentUser,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    state.auth_service.logout(request.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current authenticated user", body = crate::modules::users::dto::UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponseBody)
    )
)]
pub async fn me(
    state: web::Data<AppState>,
    current_user: CurrentUser,
) -> Result<HttpResponse, AppError> {
    let response = state.auth_service.me(&current_user.0).await?;
    Ok(HttpResponse::Ok().json(response))
}

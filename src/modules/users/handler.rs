use actix_web::{HttpResponse, web};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::app_error::AppError,
    modules::users::dto::{CreateUserRequest, UpdateUserRequest, UserQuery},
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Create a user", body = crate::modules::users::dto::UserResponse),
        (status = 400, description = "Validation error", body = ErrorResponseBody),
        (status = 403, description = "Forbidden", body = ErrorResponseBody),
        (status = 409, description = "Email already exists", body = ErrorResponseBody)
    )
)]
pub async fn create_user(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state
        .user_service
        .create_user(&current_user.0, request.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User detail", body = crate::modules::users::dto::UserResponse),
        (status = 403, description = "Forbidden", body = ErrorResponseBody),
        (status = 404, description = "User not found", body = ErrorResponseBody)
    )
)]
pub async fn get_user(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .user_service
        .get_user(&current_user.0, user_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    params(
        UserQuery
    ),
    responses(
        (status = 200, description = "List users", body = crate::modules::users::dto::PaginatedUsersResponse),
        (status = 403, description = "Forbidden", body = ErrorResponseBody)
    )
)]
pub async fn list_users(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    query: web::Query<UserQuery>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .user_service
        .list_users(&current_user.0, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/{id}",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Updated user", body = crate::modules::users::dto::UserResponse),
        (status = 400, description = "Validation error", body = ErrorResponseBody),
        (status = 403, description = "Forbidden", body = ErrorResponseBody),
        (status = 404, description = "User not found", body = ErrorResponseBody),
        (status = 409, description = "Email already exists", body = ErrorResponseBody)
    )
)]
pub async fn update_user(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    user_id: web::Path<Uuid>,
    request: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state
        .user_service
        .update_user(&current_user.0, user_id.into_inner(), request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 403, description = "Forbidden", body = ErrorResponseBody),
        (status = 404, description = "User not found", body = ErrorResponseBody)
    )
)]
pub async fn delete_user(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    state
        .user_service
        .delete_user(&current_user.0, user_id.into_inner())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

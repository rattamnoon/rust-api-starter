use actix_web::{HttpResponse, web};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::app_error::AppError,
    modules::products::dto::{CreateProductRequest, ProductQuery, UpdateProductRequest},
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    post,
    path = "/api/v1/products",
    tag = "Products",
    security(("bearer_auth" = [])),
    request_body = CreateProductRequest,
    responses(
        (status = 201, body = crate::modules::products::dto::ProductResponse),
        (status = 400, body = ErrorResponseBody),
        (status = 403, body = ErrorResponseBody),
        (status = 409, body = ErrorResponseBody)
    )
)]
pub async fn create_product(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    request: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state
        .product_service
        .create_product(&current_user.0, request.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/products",
    tag = "Products",
    params(ProductQuery),
    responses((status = 200, body = crate::modules::products::dto::ProductsListResponse))
)]
pub async fn list_products(
    state: web::Data<AppState>,
    query: web::Query<ProductQuery>,
) -> Result<HttpResponse, AppError> {
    let response = state.product_service.list_products(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    tag = "Products",
    params(("id" = Uuid, Path, description = "Product ID")),
    responses(
        (status = 200, body = crate::modules::products::dto::ProductResponse),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn get_product(
    state: web::Data<AppState>,
    product_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state.product_service.get_product(product_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    patch,
    path = "/api/v1/products/{id}",
    tag = "Products",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Product ID")),
    request_body = UpdateProductRequest,
    responses(
        (status = 200, body = crate::modules::products::dto::ProductResponse),
        (status = 400, body = ErrorResponseBody),
        (status = 403, body = ErrorResponseBody),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn update_product(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    product_id: web::Path<Uuid>,
    request: web::Json<UpdateProductRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state
        .product_service
        .update_product(&current_user.0, product_id.into_inner(), request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    delete,
    path = "/api/v1/products/{id}",
    tag = "Products",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Product ID")),
    responses(
        (status = 204, description = "Product deleted"),
        (status = 403, body = ErrorResponseBody),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn delete_product(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    product_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    state
        .product_service
        .delete_product(&current_user.0, product_id.into_inner())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

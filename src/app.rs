use actix_web::{HttpResponse, web};
use prometheus::{Encoder, TextEncoder};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    docs::ApiDoc,
    modules::{auth, jobs, orders, payments, products, receipts, uploads, users},
    shared::metrics,
    shared::response::HealthResponse,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health))
            .service(auth::routes::scope())
            .service(jobs::routes::scope())
            .service(orders::routes::scope())
            .service(payments::routes::scope())
            .service(products::routes::scope())
            .service(receipts::routes::scope())
            .service(uploads::routes::scope())
            .service(users::routes::scope()),
    )
    .route("/metrics", web::get().to(metrics_handler))
    .service(uploads::routes::static_scope())
    .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()));
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "API health check", body = HealthResponse)
    )
)]
pub(crate) async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub(crate) async fn metrics_handler() -> Result<HttpResponse, crate::errors::app_error::AppError> {
    let body = metrics::gather_metrics()?;
    Ok(HttpResponse::Ok()
        .content_type(TextEncoder::new().format_type())
        .body(body))
}

use actix_web::{HttpResponse, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    docs::ApiDoc,
    modules::{auth, uploads, users},
    shared::response::HealthResponse,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health))
            .service(auth::routes::scope())
            .service(uploads::routes::scope())
            .service(users::routes::scope()),
    )
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

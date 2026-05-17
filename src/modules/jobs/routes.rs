use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/jobs")
        .wrap(AuthMiddleware)
        .route("", web::get().to(handler::list_jobs))
        .route("/charts/summary", web::get().to(handler::chart_summary))
        .route("/{id}", web::get().to(handler::get_job))
        .route("/{id}/retry", web::post().to(handler::retry_job))
}

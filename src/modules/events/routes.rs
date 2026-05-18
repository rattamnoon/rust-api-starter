use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/events")
        .wrap(AuthMiddleware)
        .route("", web::get().to(handler::list_events))
        .route("/{id}", web::get().to(handler::get_event))
}

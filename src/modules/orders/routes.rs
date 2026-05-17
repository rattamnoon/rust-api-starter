use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/orders")
        .wrap(AuthMiddleware)
        .route("", web::post().to(handler::create_order))
        .route("", web::get().to(handler::list_orders))
        .route("/{id}", web::get().to(handler::get_order))
}

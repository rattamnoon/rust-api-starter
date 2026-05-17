use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/products")
        .route("", web::get().to(handler::list_products))
        .route("/{id}", web::get().to(handler::get_product))
        .service(
            web::scope("")
                .wrap(AuthMiddleware)
                .route("", web::post().to(handler::create_product))
                .route("/{id}", web::patch().to(handler::update_product))
                .route("/{id}", web::delete().to(handler::delete_product)),
        )
}

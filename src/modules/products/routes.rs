use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/products")
        .service(web::resource("").route(web::get().to(handler::list_products)))
        .service(web::resource("/{id}").route(web::get().to(handler::get_product)))
        .service(
            web::resource("")
                .wrap(AuthMiddleware)
                .route(web::post().to(handler::create_product)),
        )
        .service(
            web::resource("/{id}")
                .wrap(AuthMiddleware)
                .route(web::patch().to(handler::update_product))
                .route(web::delete().to(handler::delete_product)),
        )
}

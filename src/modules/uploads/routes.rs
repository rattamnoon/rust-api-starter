use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/uploads")
        .wrap(AuthMiddleware)
        .route("", web::post().to(handler::upload_file))
}

pub fn static_scope() -> impl HttpServiceFactory {
    web::scope("/static").route(
        "/{sub_folder}/{file}",
        web::get().to(handler::get_static_file),
    )
}

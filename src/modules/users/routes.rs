use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/users")
        .wrap(AuthMiddleware)
        .route("", web::post().to(handler::create_user))
        .route("", web::get().to(handler::list_users))
        .route("/{id}", web::get().to(handler::get_user))
        .route("/{id}", web::patch().to(handler::update_user))
        .route("/{id}", web::delete().to(handler::delete_user))
}

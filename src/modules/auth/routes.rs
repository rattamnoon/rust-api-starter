use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/auth")
        .route("/register", web::post().to(handler::register))
        .route("/login", web::post().to(handler::login))
        .route("/refresh", web::post().to(handler::refresh))
        .service(
            web::scope("")
                .wrap(AuthMiddleware)
                .route("/logout", web::post().to(handler::logout))
                .route("/me", web::get().to(handler::me)),
        )
}

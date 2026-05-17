use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("")
        .service(
            web::scope("/orders")
                .wrap(AuthMiddleware)
                .route("/{id}/checkout", web::post().to(handler::create_checkout_session)),
        )
        .service(
            web::scope("/payments")
                .route("/webhooks/stripe", web::post().to(handler::stripe_webhook)),
        )
}

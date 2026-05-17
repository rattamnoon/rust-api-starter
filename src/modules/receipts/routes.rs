use actix_web::{dev::HttpServiceFactory, web};

use crate::middleware::auth_middleware::AuthMiddleware;

use super::handler;

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/receipts")
        .wrap(AuthMiddleware)
        .route("/{id}", web::get().to(handler::get_receipt))
        .route("/{id}/pdf", web::get().to(handler::get_receipt_pdf))
        .route("/{id}/resend", web::post().to(handler::resend_receipt))
}

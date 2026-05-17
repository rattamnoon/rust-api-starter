use actix_files::NamedFile;
use actix_web::{
    HttpResponse,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web,
};
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    get,
    path = "/api/v1/receipts/{id}",
    tag = "Receipts",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Receipt ID")),
    responses(
        (status = 200, body = crate::modules::receipts::dto::ReceiptResponse),
        (status = 403, body = ErrorResponseBody),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn get_receipt(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    receipt_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .receipt_service
        .get_receipt(&current_user.0, receipt_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/receipts/{id}/pdf",
    tag = "Receipts",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Receipt ID")),
    responses(
        (status = 200, description = "Receipt PDF"),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn get_receipt_pdf(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    receipt_id: web::Path<Uuid>,
) -> Result<NamedFile, AppError> {
    let upload_id = state
        .receipt_service
        .get_pdf_upload_id(&current_user.0, receipt_id.into_inner())
        .await?;
    let upload = state
        .upload_service
        .find_by_id(upload_id)
        .await?;
    let named_file = NamedFile::open_async(&upload.storage_path)
        .await
        .map_err(|_| AppError::NotFound("receipt PDF was not found".into()))?
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![DispositionParam::Filename(upload.original_filename.clone())],
        });
    Ok(named_file)
}

#[utoipa::path(
    post,
    path = "/api/v1/receipts/{id}/resend",
    tag = "Receipts",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Receipt ID")),
    responses(
        (status = 200, body = crate::modules::receipts::dto::ReceiptResponse),
        (status = 403, body = ErrorResponseBody),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn resend_receipt(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    receipt_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .receipt_service
        .resend_receipt(&current_user.0, receipt_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

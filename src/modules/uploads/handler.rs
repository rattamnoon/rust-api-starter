use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{
    HttpResponse,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web,
};
use futures_util::{StreamExt, TryStreamExt};

use crate::{
    errors::app_error::AppError,
    modules::uploads::dto::{UploadFileMultipartRequest, UploadFileRequest},
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    post,
    path = "/api/v1/uploads",
    tag = "Uploads",
    security(
        ("bearer_auth" = [])
    ),
    request_body(
        content = UploadFileMultipartRequest,
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 201, description = "Stored file metadata", body = crate::modules::uploads::dto::UploadFileResponse),
        (status = 400, description = "Invalid upload payload", body = ErrorResponseBody),
        (status = 401, description = "Authentication required", body = ErrorResponseBody)
    )
)]
pub async fn upload_file(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let mut sub_folder = None;
    let mut upload_request = None;

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|error| AppError::BadRequest(format!("invalid multipart payload: {error}")))?
    {
        match field.name() {
            Some("sub_folder") => {
                let mut bytes = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|error| {
                    AppError::BadRequest(format!("failed to read `sub_folder`: {error}"))
                })? {
                    bytes.extend_from_slice(&chunk);
                }

                let value = String::from_utf8(bytes)
                    .map_err(|_| AppError::BadRequest("`sub_folder` must be valid UTF-8".into()))?;
                sub_folder = Some(value);
            }
            Some("file") => {
                let original_filename = field
                    .content_disposition()
                    .and_then(|disposition| disposition.get_filename().map(ToString::to_string))
                    .ok_or_else(|| {
                        AppError::BadRequest("multipart field `file` requires a filename".into())
                    })?;
                let content_type = field.content_type().map(ToString::to_string);
                let mut bytes = Vec::new();

                while let Some(chunk) = field.try_next().await.map_err(|error| {
                    AppError::BadRequest(format!("failed to read uploaded file: {error}"))
                })? {
                    bytes.extend_from_slice(&chunk);
                }

                upload_request = Some(UploadFileRequest {
                    sub_folder: sub_folder.clone(),
                    original_filename,
                    content_type,
                    bytes,
                });
            }
            _ => while field.next().await.is_some() {},
        }
    }

    let mut upload_request = upload_request
        .ok_or_else(|| AppError::BadRequest("multipart field `file` is required".into()))?;
    if upload_request.sub_folder.is_none() {
        upload_request.sub_folder = sub_folder;
    }

    let response = state
        .upload_service
        .upload_file(&current_user.0, upload_request)
        .await?;

    Ok(HttpResponse::Created().json(response))
}

#[utoipa::path(
    get,
    path = "/static/{sub_folder}/{file}",
    tag = "Uploads",
    params(
        ("sub_folder" = String, Path, description = "Upload sub folder"),
        ("file" = String, Path, description = "Generated stored filename")
    ),
    responses(
        (status = 200, description = "Static file content"),
        (status = 404, description = "File not found", body = ErrorResponseBody)
    )
)]
pub async fn get_static_file(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<NamedFile, AppError> {
    let (sub_folder, stored_filename) = path.into_inner();
    let file_record = state
        .upload_service
        .find_file_for_static(&sub_folder, &stored_filename)
        .await?;

    let named_file = NamedFile::open_async(&file_record.storage_path)
        .await
        .map_err(|_| AppError::NotFound("file was not found".into()))?
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![DispositionParam::Filename(
                file_record.original_filename.clone(),
            )],
        });

    Ok(named_file)
}

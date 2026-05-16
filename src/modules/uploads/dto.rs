use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::modules::uploads::model::UploadedFile;

#[derive(Debug, ToSchema)]
pub struct UploadFileMultipartRequest {
    #[schema(example = "images")]
    pub sub_folder: Option<String>,
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

#[derive(Debug)]
pub struct UploadFileRequest {
    pub sub_folder: Option<String>,
    pub original_filename: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadFileResponse {
    pub id: Uuid,
    pub sub_folder: String,
    pub original_filename: String,
    pub stored_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
    pub storage_path: String,
    pub public_url: String,
    pub uploaded_by: Uuid,
    pub uploaded_at: DateTime<Utc>,
}

impl UploadFileResponse {
    pub fn from_model(file: UploadedFile) -> Self {
        Self {
            id: file.id,
            sub_folder: file.sub_folder.clone(),
            original_filename: file.original_filename,
            stored_filename: file.stored_filename.clone(),
            content_type: file.content_type,
            size_bytes: file.size_bytes,
            storage_path: file.storage_path,
            public_url: format!("/static/{}/{}", file.sub_folder, file.stored_filename),
            uploaded_by: file.uploaded_by,
            uploaded_at: file.created_at,
        }
    }
}

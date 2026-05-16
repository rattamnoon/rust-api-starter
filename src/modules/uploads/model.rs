use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UploadedFile {
    pub id: Uuid,
    pub sub_folder: String,
    pub original_filename: String,
    pub stored_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
    pub storage_path: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

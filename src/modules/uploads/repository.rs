use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::uploads::model::UploadedFile;

#[derive(Clone)]
pub struct UploadRepository {
    pool: PgPool,
}

#[derive(Debug)]
pub struct NewUploadedFile {
    pub sub_folder: String,
    pub original_filename: String,
    pub stored_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
    pub storage_path: String,
    pub uploaded_by: Uuid,
}

impl UploadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: NewUploadedFile) -> Result<UploadedFile, sqlx::Error> {
        sqlx::query_as::<_, UploadedFile>(
            "INSERT INTO uploaded_files (
                sub_folder,
                original_filename,
                stored_filename,
                content_type,
                size_bytes,
                storage_path,
                uploaded_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, sub_folder, original_filename, stored_filename, content_type, size_bytes, storage_path, uploaded_by, created_at",
        )
        .bind(input.sub_folder)
        .bind(input.original_filename)
        .bind(input.stored_filename)
        .bind(input.content_type)
        .bind(input.size_bytes)
        .bind(input.storage_path)
        .bind(input.uploaded_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_public_path(
        &self,
        sub_folder: &str,
        stored_filename: &str,
    ) -> Result<Option<UploadedFile>, sqlx::Error> {
        sqlx::query_as::<_, UploadedFile>(
            "SELECT id, sub_folder, original_filename, stored_filename, content_type, size_bytes, storage_path, uploaded_by, created_at
             FROM uploaded_files
             WHERE sub_folder = $1 AND stored_filename = $2",
        )
        .bind(sub_folder)
        .bind(stored_filename)
        .fetch_optional(&self.pool)
        .await
    }
}

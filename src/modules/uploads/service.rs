use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::uploads::{
        dto::{UploadFileRequest, UploadFileResponse},
        model::UploadedFile,
        repository::{NewUploadedFile, UploadRepository},
    },
    shared::{extractor::AuthenticatedUser, file_storage::LocalFileStorage},
};

const DEFAULT_SUB_FOLDER: &str = "general";
const MAX_UPLOAD_SIZE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone)]
pub struct UploadService {
    repository: UploadRepository,
    file_storage: LocalFileStorage,
}

impl UploadService {
    pub fn new(repository: UploadRepository, file_storage: LocalFileStorage) -> Self {
        Self {
            repository,
            file_storage,
        }
    }

    pub async fn upload_file(
        &self,
        actor: &AuthenticatedUser,
        request: UploadFileRequest,
    ) -> Result<UploadFileResponse, AppError> {
        let sub_folder = sanitize_sub_folder(request.sub_folder.as_deref())?;

        if request.original_filename.trim().is_empty() {
            return Err(AppError::BadRequest(
                "uploaded file must include a filename".into(),
            ));
        }

        if request.bytes.is_empty() {
            return Err(AppError::BadRequest("uploaded file is empty".into()));
        }

        if request.bytes.len() > MAX_UPLOAD_SIZE_BYTES {
            return Err(AppError::BadRequest(format!(
                "uploaded file exceeds the {} MB limit",
                MAX_UPLOAD_SIZE_BYTES / 1024 / 1024
            )));
        }

        let stored_filename = build_stored_filename(&request.original_filename);
        let stored_path = self
            .file_storage
            .store(&sub_folder, &stored_filename, &request.bytes)
            .await?;

        let model = match self
            .repository
            .create(NewUploadedFile {
                sub_folder: sub_folder.clone(),
                original_filename: sanitize_original_filename(&request.original_filename),
                stored_filename: stored_filename.clone(),
                content_type: request.content_type,
                size_bytes: request.bytes.len() as i64,
                storage_path: stored_path.to_string_lossy().to_string(),
                uploaded_by: actor.user_id,
            })
            .await
        {
            Ok(model) => model,
            Err(error) => {
                self.file_storage.delete(&stored_path).await?;
                return Err(error.into());
            }
        };

        Ok(UploadFileResponse::from_model(model))
    }

    pub async fn find_file_for_static(
        &self,
        sub_folder: &str,
        stored_filename: &str,
    ) -> Result<UploadedFile, AppError> {
        let normalized_folder = sanitize_sub_folder(Some(sub_folder))?;
        let normalized_file = sanitize_public_filename(stored_filename)?;

        let model = self
            .repository
            .find_by_public_path(&normalized_folder, &normalized_file)
            .await?
            .ok_or_else(|| AppError::NotFound("file was not found".into()))?;

        let resolved_path = PathBuf::from(&model.storage_path);
        if !resolved_path.starts_with(self.file_storage.root_dir()) {
            return Err(AppError::Internal(
                "stored file path resolves outside the upload directory".into(),
            ));
        }

        Ok(model)
    }
}

fn sanitize_sub_folder(input: Option<&str>) -> Result<String, AppError> {
    let raw = input.unwrap_or(DEFAULT_SUB_FOLDER).trim();
    if raw.is_empty() {
        return Ok(DEFAULT_SUB_FOLDER.to_string());
    }

    let sanitized: String = raw
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else if matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();

    let compact = sanitized
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if compact.is_empty() {
        return Err(AppError::BadRequest(
            "sub_folder must contain at least one alphanumeric character".into(),
        ));
    }

    if compact.len() > 64 {
        return Err(AppError::BadRequest(
            "sub_folder must be 64 characters or fewer".into(),
        ));
    }

    Ok(compact)
}

fn sanitize_original_filename(original_filename: &str) -> String {
    Path::new(original_filename)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("file")
        .to_string()
}

fn build_stored_filename(original_filename: &str) -> String {
    let stem = Uuid::now_v7().to_string();
    match safe_extension(original_filename) {
        Some(extension) => format!("{stem}.{extension}"),
        None => stem,
    }
}

fn safe_extension(original_filename: &str) -> Option<String> {
    let extension = Path::new(original_filename)
        .extension()
        .and_then(|value| value.to_str())?
        .trim()
        .to_ascii_lowercase();

    if extension.is_empty() {
        return None;
    }

    let sanitized: String = extension
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(16)
        .collect();

    (!sanitized.is_empty()).then_some(sanitized)
}

fn sanitize_public_filename(stored_filename: &str) -> Result<String, AppError> {
    let raw = stored_filename.trim();
    if raw.is_empty() {
        return Err(AppError::NotFound("file was not found".into()));
    }

    if raw.contains('/') || raw.contains('\\') || raw.contains("..") {
        return Err(AppError::NotFound("file was not found".into()));
    }

    let sanitized: String = raw
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
        .collect();

    if sanitized != raw {
        return Err(AppError::NotFound("file was not found".into()));
    }

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::{
        build_stored_filename, safe_extension, sanitize_public_filename, sanitize_sub_folder,
    };

    #[test]
    fn keeps_safe_extension() {
        let filename = build_stored_filename("avatar.PNG");
        assert!(filename.ends_with(".png"));
    }

    #[test]
    fn removes_unsafe_extension_characters() {
        assert_eq!(safe_extension("archive.t@r.gz"), Some("gz".to_string()));
        assert_eq!(safe_extension("payload.%%%"), None);
    }

    #[test]
    fn normalizes_sub_folder() {
        assert_eq!(
            sanitize_sub_folder(Some("Product Images")).unwrap(),
            "product-images"
        );
    }

    #[test]
    fn rejects_traversal_in_public_filename() {
        assert!(sanitize_public_filename("../secret.txt").is_err());
    }
}

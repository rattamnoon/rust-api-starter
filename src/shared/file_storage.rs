use std::path::{Path, PathBuf};

use tokio::fs;

use crate::errors::app_error::AppError;

#[derive(Clone)]
pub struct LocalFileStorage {
    root_dir: PathBuf,
}

impl LocalFileStorage {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn absolute_path(&self, sub_folder: &str, stored_filename: &str) -> PathBuf {
        self.root_dir.join(sub_folder).join(stored_filename)
    }

    pub async fn store(
        &self,
        sub_folder: &str,
        stored_filename: &str,
        bytes: &[u8],
    ) -> Result<PathBuf, AppError> {
        let directory = self.root_dir.join(sub_folder);
        fs::create_dir_all(&directory).await.map_err(|error| {
            AppError::Internal(format!("failed to create upload directory: {error}"))
        })?;

        let path = directory.join(stored_filename);
        fs::write(&path, bytes).await.map_err(|error| {
            AppError::Internal(format!("failed to write uploaded file: {error}"))
        })?;
        Ok(path)
    }

    pub async fn delete(&self, path: &Path) -> Result<(), AppError> {
        match fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(AppError::Internal(format!(
                "failed to delete uploaded file after database error: {error}"
            ))),
        }
    }
}

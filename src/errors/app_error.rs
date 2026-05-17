use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde_json::{Value, json};
use thiserror::Error;
use validator::ValidationErrors;

use crate::shared::response::{ErrorDetails, ErrorResponseBody};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("validation failed")]
    Validation(ValidationErrors),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Config(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("token error")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("queue error")]
    Queue(#[from] lapin::Error),
    #[error("http client error")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    PasswordHash(String),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::Validation(_) => "validation_error",
            Self::NotFound(_) => "not_found",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::Conflict(_) => "conflict",
            Self::Config(_) => "config_error",
            Self::Database(_) => "database_error",
            Self::Jwt(_) => "token_error",
            Self::Queue(_) => "queue_error",
            Self::Http(_) => "http_error",
            Self::PasswordHash(_) => "password_error",
            Self::Io(_) => "io_error",
            Self::Internal(_) => "internal_error",
        }
    }

    fn details(&self) -> Option<Value> {
        match self {
            Self::Validation(errors) => Some(validation_errors_to_json(errors)),
            _ => None,
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) | Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Config(_)
            | Self::Database(_)
            | Self::Queue(_)
            | Self::Http(_)
            | Self::PasswordHash(_)
            | Self::Io(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponseBody {
            error: ErrorDetails {
                code: self.code().to_string(),
                message: self.to_string(),
                details: self.details(),
            },
        })
    }
}

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        Self::Validation(value)
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(value.to_string())
    }
}

fn validation_errors_to_json(errors: &ValidationErrors) -> Value {
    let mut fields = serde_json::Map::new();

    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|error| {
                error
                    .message
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "invalid value".to_string())
            })
            .collect();

        fields.insert(field.to_string(), json!(messages));
    }

    Value::Object(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_maps_to_401() {
        let error = AppError::Unauthorized("invalid token".into());
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }
}

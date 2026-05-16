use std::env;

use crate::errors::app_error::AppError;

#[derive(Clone, Debug)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: i64,
    pub jwt_refresh_expires_in: i64,
    pub log_dir: String,
    pub upload_dir: String,
    pub rate_limit_requests: usize,
    pub rate_limit_window_seconds: u64,
    pub rust_log: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Settings {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            database_url: read_required("DATABASE_URL")?,
            jwt_secret: read_required("JWT_SECRET")?,
            jwt_expires_in: read_required("JWT_EXPIRES_IN")?
                .parse()
                .map_err(|_| AppError::Config("JWT_EXPIRES_IN must be a valid integer".into()))?,
            jwt_refresh_expires_in: read_required("JWT_REFRESH_EXPIRES_IN")?.parse().map_err(
                |_| AppError::Config("JWT_REFRESH_EXPIRES_IN must be a valid integer".into()),
            )?,
            log_dir: read_optional("LOG_DIR", "./logs"),
            upload_dir: read_optional("UPLOAD_DIR", "./uploads"),
            rate_limit_requests: read_optional_parse("RATE_LIMIT_REQUESTS", 60)?,
            rate_limit_window_seconds: read_optional_parse("RATE_LIMIT_WINDOW_SECONDS", 60)?,
            rust_log: read_optional("RUST_LOG", "info,actix_web=info"),
            server_host: read_required("SERVER_HOST")?,
            server_port: read_required("SERVER_PORT")?
                .parse()
                .map_err(|_| AppError::Config("SERVER_PORT must be a valid integer".into()))?,
        })
    }
}

fn read_required(key: &str) -> Result<String, AppError> {
    env::var(key)
        .map_err(|_| AppError::Config(format!("missing required environment variable: {key}")))
}

fn read_optional(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn read_optional_parse<T>(key: &str, default: T) -> Result<T, AppError>
where
    T: std::str::FromStr + ToString + Copy,
{
    match env::var(key) {
        Ok(value) => value
            .parse()
            .map_err(|_| AppError::Config(format!("{key} must be a valid integer"))),
        Err(_) => Ok(default),
    }
}

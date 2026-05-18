use std::env;

use crate::errors::app_error::AppError;

#[derive(Clone, Debug)]
pub struct Settings {
    pub database_url: String,
    pub rabbitmq_url: String,
    pub rabbitmq_queue_name: String,
    pub rabbitmq_dead_letter_queue: String,
    pub kafka_brokers: String,
    pub kafka_client_id: String,
    pub kafka_topic_users: String,
    pub kafka_topic_orders: String,
    pub kafka_topic_receipts: String,
    pub worker_concurrency: u16,
    pub job_max_retries: i32,
    pub jwt_secret: String,
    pub jwt_expires_in: i64,
    pub jwt_refresh_expires_in: i64,
    pub public_base_url: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub stripe_success_url: String,
    pub stripe_cancel_url: String,
    pub email_provider: String,
    pub email_from: String,
    pub resend_api_key: String,
    pub receipt_prefix: String,
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
            rabbitmq_url: read_required("RABBITMQ_URL")?,
            rabbitmq_queue_name: read_optional("RABBITMQ_QUEUE_NAME", "jobs"),
            rabbitmq_dead_letter_queue: read_optional("RABBITMQ_DEAD_LETTER_QUEUE", "jobs.dead"),
            kafka_brokers: read_required("KAFKA_BROKERS")?,
            kafka_client_id: read_optional("KAFKA_CLIENT_ID", "rust-api-starter"),
            kafka_topic_users: read_optional("KAFKA_TOPIC_USERS", "users.events"),
            kafka_topic_orders: read_optional("KAFKA_TOPIC_ORDERS", "orders.events"),
            kafka_topic_receipts: read_optional("KAFKA_TOPIC_RECEIPTS", "receipts.events"),
            worker_concurrency: read_optional_parse("WORKER_CONCURRENCY", 8)?,
            job_max_retries: read_optional_parse("JOB_MAX_RETRIES", 3)?,
            jwt_secret: read_required("JWT_SECRET")?,
            jwt_expires_in: read_required("JWT_EXPIRES_IN")?
                .parse()
                .map_err(|_| AppError::Config("JWT_EXPIRES_IN must be a valid integer".into()))?,
            jwt_refresh_expires_in: read_required("JWT_REFRESH_EXPIRES_IN")?.parse().map_err(
                |_| AppError::Config("JWT_REFRESH_EXPIRES_IN must be a valid integer".into()),
            )?,
            public_base_url: read_optional("PUBLIC_BASE_URL", "http://127.0.0.1:8080"),
            stripe_secret_key: read_optional("STRIPE_SECRET_KEY", ""),
            stripe_webhook_secret: read_optional("STRIPE_WEBHOOK_SECRET", ""),
            stripe_success_url: read_optional(
                "STRIPE_SUCCESS_URL",
                "http://127.0.0.1:8080/api/v1/orders/success",
            ),
            stripe_cancel_url: read_optional(
                "STRIPE_CANCEL_URL",
                "http://127.0.0.1:8080/api/v1/orders/cancel",
            ),
            email_provider: read_optional("EMAIL_PROVIDER", "log"),
            email_from: read_optional("EMAIL_FROM", "noreply@example.com"),
            resend_api_key: read_optional("RESEND_API_KEY", ""),
            receipt_prefix: read_optional("RECEIPT_PREFIX", "RCT"),
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

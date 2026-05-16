use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponseBody {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

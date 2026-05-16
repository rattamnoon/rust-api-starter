use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{errors::app_error::AppError, modules::users::model::User, shared::types::UserRole};

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email(message = "email must be valid"))]
    pub email: String,
    #[validate(length(min = 3, message = "name must be at least 3 characters"))]
    pub name: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(email(message = "email must be valid"))]
    pub email: Option<String>,
    #[validate(length(min = 3, message = "name must be at least 3 characters"))]
    pub name: Option<String>,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct UserQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedUsersResponse {
    pub items: Vec<UserResponse>,
    pub page: i64,
    pub limit: i64,
}

impl UserResponse {
    pub fn from_model(user: User) -> Result<Self, AppError> {
        let role = user.role()?;

        Ok(Self {
            id: user.id,
            email: user.email,
            name: user.name,
            role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }
}

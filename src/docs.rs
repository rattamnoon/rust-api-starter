use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

use crate::{
    app,
    modules::{auth, uploads, users},
    shared::response::{ErrorDetails, ErrorResponseBody, HealthResponse},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        app::health,
        auth::handler::register,
        auth::handler::login,
        auth::handler::refresh,
        auth::handler::logout,
        auth::handler::me,
        uploads::handler::upload_file,
        users::handler::create_user,
        users::handler::get_user,
        users::handler::list_users,
        users::handler::update_user,
        users::handler::delete_user
    ),
    components(
        schemas(
            HealthResponse,
            ErrorResponseBody,
            ErrorDetails,
            auth::dto::RegisterRequest,
            auth::dto::LoginRequest,
            auth::dto::RefreshTokenRequest,
            auth::dto::AuthResponse,
            uploads::dto::UploadFileMultipartRequest,
            uploads::dto::UploadFileResponse,
            users::dto::CreateUserRequest,
            users::dto::UpdateUserRequest,
            users::dto::UserQuery,
            users::dto::UserResponse,
            users::dto::PaginatedUsersResponse,
            crate::shared::types::UserRole
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Auth", description = "Authentication and current-user endpoints"),
        (name = "Uploads", description = "Local file upload endpoints"),
        (name = "Users", description = "User CRUD endpoints")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

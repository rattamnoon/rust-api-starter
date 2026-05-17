use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

use crate::{
    app,
    modules::{auth, jobs, orders, payments, products, receipts, uploads, users},
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
        jobs::handler::list_jobs,
        jobs::handler::get_job,
        jobs::handler::retry_job,
        jobs::handler::chart_summary,
        orders::handler::create_order,
        orders::handler::list_orders,
        orders::handler::get_order,
        payments::handler::create_checkout_session,
        payments::handler::stripe_webhook,
        products::handler::create_product,
        products::handler::list_products,
        products::handler::get_product,
        products::handler::update_product,
        products::handler::delete_product,
        receipts::handler::get_receipt,
        receipts::handler::get_receipt_pdf,
        receipts::handler::resend_receipt,
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
            jobs::dto::JobsQuery,
            jobs::dto::JobResponse,
            jobs::dto::JobAttemptResponse,
            jobs::dto::JobDetailResponse,
            jobs::dto::JobsListResponse,
            jobs::dto::ChartPoint,
            jobs::dto::TimelinePoint,
            jobs::dto::DurationPoint,
            jobs::dto::JobsChartSummaryResponse,
            orders::dto::CreateOrderItemRequest,
            orders::dto::CreateOrderRequest,
            orders::dto::OrderQuery,
            orders::dto::OrderItemResponse,
            orders::dto::OrderResponse,
            orders::dto::OrdersListResponse,
            orders::model::OrderStatus,
            payments::dto::CheckoutOrderRequest,
            payments::dto::CheckoutSessionResponse,
            payments::dto::PaymentWebhookAcceptedResponse,
            payments::model::PaymentStatus,
            products::dto::CreateProductRequest,
            products::dto::UpdateProductRequest,
            products::dto::ProductQuery,
            products::dto::ProductResponse,
            products::dto::ProductsListResponse,
            products::model::Currency,
            receipts::dto::ReceiptResponse,
            receipts::model::ReceiptStatus,
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
        (name = "Jobs", description = "Background job and chart endpoints"),
        (name = "Orders", description = "Order creation and lookup endpoints"),
        (name = "Payments", description = "Stripe checkout and webhook endpoints"),
        (name = "Products", description = "Product catalog endpoints"),
        (name = "Receipts", description = "Receipt metadata, PDF, and resend endpoints"),
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

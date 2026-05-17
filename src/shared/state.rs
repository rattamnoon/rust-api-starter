use sqlx::PgPool;

use crate::{
    config::settings::Settings,
    modules::{
        auth::{repository::AuthRepository, service::AuthService},
        jobs::{repository::JobRepository, service::JobService},
        orders::{repository::OrderRepository, service::OrderService},
        payments::{repository::PaymentRepository, service::PaymentService},
        products::{repository::ProductRepository, service::ProductService},
        receipts::{repository::ReceiptRepository, service::ReceiptService},
        uploads::{repository::UploadRepository, service::UploadService},
        users::{repository::UserRepository, service::UserService},
    },
    shared::{
        email::EmailService, file_storage::LocalFileStorage, jwt::JwtService,
        password::PasswordService, queue::RabbitMqClient, rate_limit::RateLimiter,
        temporal::TemporalCommerceService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub jwt_service: JwtService,
    pub auth_service: AuthService,
    pub job_service: JobService,
    pub order_service: OrderService,
    pub payment_service: PaymentService,
    pub product_service: ProductService,
    pub rate_limiter: RateLimiter,
    pub receipt_service: ReceiptService,
    pub upload_service: UploadService,
    pub user_service: UserService,
}

impl AppState {
    pub async fn new(
        settings: Settings,
        db: PgPool,
    ) -> Result<Self, crate::errors::app_error::AppError> {
        let jwt_service = JwtService::new(
            settings.jwt_secret.clone(),
            settings.jwt_expires_in,
            settings.jwt_refresh_expires_in,
        );
        let password_service = PasswordService::new();
        let rate_limiter = RateLimiter::new(
            settings.rate_limit_requests,
            std::time::Duration::from_secs(settings.rate_limit_window_seconds),
        );

        let auth_repository = AuthRepository::new(db.clone());
        let job_repository = JobRepository::new(db.clone());
        let order_repository = OrderRepository::new(db.clone());
        let payment_repository = PaymentRepository::new(db.clone());
        let product_repository = ProductRepository::new(db.clone());
        let receipt_repository = ReceiptRepository::new(db.clone());
        let upload_repository = UploadRepository::new(db.clone());
        let user_repository = UserRepository::new(db.clone());
        let email_service = EmailService::new(&settings);
        let file_storage = LocalFileStorage::new(settings.upload_dir.clone().into());
        let queue = RabbitMqClient::connect(
            &settings.rabbitmq_url,
            &settings.rabbitmq_queue_name,
            &settings.rabbitmq_dead_letter_queue,
        )
        .await?;

        let job_service = JobService::new(job_repository, queue.clone(), settings.job_max_retries);
        let auth_service = AuthService::new(
            auth_repository,
            job_service.clone(),
            password_service.clone(),
            jwt_service.clone(),
        );
        let upload_service =
            UploadService::new(upload_repository.clone(), file_storage.clone(), job_service.clone());
        let user_service = UserService::new(user_repository.clone(), password_service);
        let product_service = ProductService::new(product_repository.clone());
        let order_service = OrderService::new(order_repository.clone(), product_repository);
        let receipt_service = ReceiptService::new(
            settings.clone(),
            receipt_repository,
            order_repository.clone(),
            upload_repository,
            user_repository,
            email_service,
            file_storage,
        );
        let temporal_service = TemporalCommerceService::new(
            &settings,
            order_repository.clone(),
            payment_repository.clone(),
            receipt_service.clone(),
        );
        let payment_service = PaymentService::new(
            settings.clone(),
            order_repository,
            payment_repository,
            temporal_service,
        );

        Ok(Self {
            jwt_service,
            auth_service,
            job_service,
            order_service,
            payment_service,
            product_service,
            rate_limiter,
            receipt_service,
            upload_service,
            user_service,
        })
    }
}

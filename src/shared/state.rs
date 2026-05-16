use sqlx::PgPool;

use crate::{
    config::settings::Settings,
    modules::{
        auth::{repository::AuthRepository, service::AuthService},
        uploads::{repository::UploadRepository, service::UploadService},
        users::{repository::UserRepository, service::UserService},
    },
    shared::{
        file_storage::LocalFileStorage, jwt::JwtService, password::PasswordService,
        rate_limit::RateLimiter,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub jwt_service: JwtService,
    pub auth_service: AuthService,
    pub rate_limiter: RateLimiter,
    pub upload_service: UploadService,
    pub user_service: UserService,
}

impl AppState {
    pub fn new(settings: Settings, db: PgPool) -> Self {
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
        let upload_repository = UploadRepository::new(db.clone());
        let user_repository = UserRepository::new(db.clone());
        let file_storage = LocalFileStorage::new(settings.upload_dir.clone().into());

        let auth_service = AuthService::new(
            auth_repository,
            password_service.clone(),
            jwt_service.clone(),
        );
        let upload_service = UploadService::new(upload_repository, file_storage);
        let user_service = UserService::new(user_repository, password_service);

        Self {
            jwt_service,
            auth_service,
            rate_limiter,
            upload_service,
            user_service,
        }
    }
}

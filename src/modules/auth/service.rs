use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::{
    errors::app_error::AppError,
    modules::{
        auth::{
            dto::{AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest},
            repository::AuthRepository,
        },
        events::service::EventService,
        jobs::service::JobService,
        users::dto::UserResponse,
    },
    shared::{
        extractor::AuthenticatedUser,
        jwt::{JwtService, TokenBundle, TokenType},
        password::PasswordService,
        types::UserRole,
    },
};

#[derive(Clone)]
pub struct AuthService {
    repository: AuthRepository,
    event_service: EventService,
    job_service: JobService,
    password_service: PasswordService,
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(
        repository: AuthRepository,
        event_service: EventService,
        job_service: JobService,
        password_service: PasswordService,
        jwt_service: JwtService,
    ) -> Self {
        Self {
            repository,
            event_service,
            job_service,
            password_service,
            jwt_service,
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AppError> {
        if self
            .repository
            .find_user_by_email(&request.email)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("email is already registered".into()));
        }

        let password_hash = self.password_service.hash_password(&request.password)?;
        let user = self
            .repository
            .create_user(
                &request.email,
                &password_hash,
                &request.name,
                UserRole::User,
            )
            .await?;

        let _ = self
            .job_service
            .enqueue_welcome_email(&user, Some(user.id))
            .await?;
        let _ = self.event_service.record_user_registered(&user).await?;

        self.issue_tokens(user).await
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = self
            .repository
            .find_user_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid email or password".into()))?;

        let password_valid = self
            .password_service
            .verify_password(&request.password, &user.password_hash)?;

        if !password_valid {
            return Err(AppError::Unauthorized("invalid email or password".into()));
        }

        self.issue_tokens(user).await
    }

    pub async fn refresh(&self, request: RefreshTokenRequest) -> Result<AuthResponse, AppError> {
        let claims = self.jwt_service.decode_token(&request.refresh_token)?;
        if claims.token_type != TokenType::Refresh {
            return Err(AppError::Unauthorized("refresh token required".into()));
        }

        let token_hash = hash_refresh_token(&request.refresh_token);
        let stored_token = self
            .repository
            .find_refresh_token(&token_hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("refresh token is invalid or expired".into()))?;

        if stored_token.user_id != claims.sub || stored_token.expires_at < Utc::now() {
            return Err(AppError::Unauthorized(
                "refresh token is invalid or expired".into(),
            ));
        }

        self.repository.revoke_refresh_token(&token_hash).await?;

        let user = self
            .repository
            .find_user_by_id(claims.sub)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user does not exist anymore".into()))?;

        self.issue_tokens(user).await
    }

    pub async fn logout(&self, request: RefreshTokenRequest) -> Result<(), AppError> {
        let token_hash = hash_refresh_token(&request.refresh_token);
        self.repository.revoke_refresh_token(&token_hash).await?;
        Ok(())
    }

    pub async fn me(&self, current_user: &AuthenticatedUser) -> Result<UserResponse, AppError> {
        let user = self
            .repository
            .find_user_by_id(current_user.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user was not found".into()))?;

        UserResponse::from_model(user)
    }

    async fn issue_tokens(
        &self,
        user: crate::modules::users::model::User,
    ) -> Result<AuthResponse, AppError> {
        let role = user.role()?;
        let TokenBundle {
            access_token,
            refresh_token,
            refresh_expires_at,
        } = self
            .jwt_service
            .generate_token_pair(user.id, &user.email, role)?;

        let refresh_token_hash = hash_refresh_token(&refresh_token);
        self.repository
            .store_refresh_token(user.id, &refresh_token_hash, refresh_expires_at)
            .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: UserResponse::from_model(user)?,
        })
    }
}

fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

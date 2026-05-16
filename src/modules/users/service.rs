use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::users::{
        dto::{
            CreateUserRequest, PaginatedUsersResponse, UpdateUserRequest, UserQuery, UserResponse,
        },
        repository::{UserRepository, UserUpdate},
    },
    shared::{extractor::AuthenticatedUser, password::PasswordService, types::UserRole},
};

#[derive(Clone)]
pub struct UserService {
    repository: UserRepository,
    password_service: PasswordService,
}

impl UserService {
    pub fn new(repository: UserRepository, password_service: PasswordService) -> Self {
        Self {
            repository,
            password_service,
        }
    }

    pub async fn create_user(
        &self,
        actor: &AuthenticatedUser,
        request: CreateUserRequest,
    ) -> Result<UserResponse, AppError> {
        ensure_admin(actor)?;

        if self
            .repository
            .find_by_email(&request.email)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("email is already registered".into()));
        }

        let password_hash = self.password_service.hash_password(&request.password)?;
        let user = self
            .repository
            .create(
                &request.email,
                &request.name,
                &password_hash,
                request.role.unwrap_or(UserRole::User),
            )
            .await?;

        UserResponse::from_model(user)
    }

    pub async fn get_user(
        &self,
        actor: &AuthenticatedUser,
        id: Uuid,
    ) -> Result<UserResponse, AppError> {
        ensure_self_or_admin(actor, id)?;
        let user = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user was not found".into()))?;

        UserResponse::from_model(user)
    }

    pub async fn list_users(
        &self,
        actor: &AuthenticatedUser,
        query: UserQuery,
    ) -> Result<PaginatedUsersResponse, AppError> {
        ensure_admin(actor)?;
        let page = query.page.max(1);
        let limit = query.limit.clamp(1, 100);

        let users = self.repository.list(page, limit).await?;
        let items = users
            .into_iter()
            .map(UserResponse::from_model)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedUsersResponse { items, page, limit })
    }

    pub async fn update_user(
        &self,
        actor: &AuthenticatedUser,
        id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        ensure_self_or_admin(actor, id)?;

        if request.role.is_some() && actor.role != UserRole::Admin {
            return Err(AppError::Forbidden("only admins can change roles".into()));
        }

        if let Some(email) = request.email.as_deref()
            && let Some(existing) = self.repository.find_by_email(email).await?
            && existing.id != id
        {
            return Err(AppError::Conflict("email is already registered".into()));
        }

        let password_hash = match request.password {
            Some(password) => Some(self.password_service.hash_password(&password)?),
            None => None,
        };

        let updated = self
            .repository
            .update(
                id,
                UserUpdate {
                    email: request.email,
                    name: request.name,
                    password_hash,
                    role: request.role,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound("user was not found".into()))?;

        UserResponse::from_model(updated)
    }

    pub async fn delete_user(&self, actor: &AuthenticatedUser, id: Uuid) -> Result<(), AppError> {
        ensure_admin(actor)?;

        let deleted = self.repository.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound("user was not found".into()));
        }

        Ok(())
    }
}

fn ensure_admin(actor: &AuthenticatedUser) -> Result<(), AppError> {
    if actor.role == UserRole::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin access is required".into()))
    }
}

fn ensure_self_or_admin(actor: &AuthenticatedUser, user_id: Uuid) -> Result<(), AppError> {
    if actor.role == UserRole::Admin || actor.user_id == user_id {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "you can only access your own user record".into(),
        ))
    }
}

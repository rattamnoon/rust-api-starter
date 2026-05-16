use std::{
    future::{Ready, ready},
    ops::Deref,
};

use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use uuid::Uuid;

use crate::{errors::app_error::AppError, shared::types::UserRole};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct CurrentUser(pub AuthenticatedUser);

impl Deref for CurrentUser {
    type Target = AuthenticatedUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for CurrentUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let user = req
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("authentication required".into()));

        ready(user.map(CurrentUser))
    }
}

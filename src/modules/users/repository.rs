use sqlx::{PgPool, QueryBuilder, postgres::Postgres};
use uuid::Uuid;

use crate::{modules::users::model::User, shared::types::UserRole};

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

#[derive(Debug, Default)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub name: Option<String>,
    pub password_hash: Option<String>,
    pub role: Option<UserRole>,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (email, name, password_hash, role)
             VALUES ($1, $2, $3, $4)
             RETURNING id, email, password_hash, name, role, created_at, updated_at",
        )
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .bind(role.as_str())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, role, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, role, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(&self, page: i64, limit: i64) -> Result<Vec<User>, sqlx::Error> {
        let offset = (page - 1).max(0) * limit;
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, role, created_at, updated_at
             FROM users
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, update: UserUpdate) -> Result<Option<User>, sqlx::Error> {
        let has_changes = update.email.is_some()
            || update.name.is_some()
            || update.password_hash.is_some()
            || update.role.is_some();

        if !has_changes {
            return self.find_by_id(id).await;
        }

        let mut builder = QueryBuilder::<Postgres>::new("UPDATE users SET ");
        let mut separated = builder.separated(", ");

        if let Some(email) = update.email {
            separated.push("email = ").push_bind(email);
        }
        if let Some(name) = update.name {
            separated.push("name = ").push_bind(name);
        }
        if let Some(password_hash) = update.password_hash {
            separated.push("password_hash = ").push_bind(password_hash);
        }
        if let Some(role) = update.role {
            separated.push("role = ").push_bind(role.as_str());
        }

        let _ = separated;
        builder
            .push(" WHERE id = ")
            .push_bind(id)
            .push(" RETURNING id, email, password_hash, name, role, created_at, updated_at");

        builder
            .build_query_as::<User>()
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let rows_affected = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(rows_affected > 0)
    }
}

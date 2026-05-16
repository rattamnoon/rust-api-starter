use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct RefreshToken {
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

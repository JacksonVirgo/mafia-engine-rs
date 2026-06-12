use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub user_id: u64,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

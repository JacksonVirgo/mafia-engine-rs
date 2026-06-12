use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct Member {
    pub user_id: u64,
    pub server_id: u64,
    pub username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

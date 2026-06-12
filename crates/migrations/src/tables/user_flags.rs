use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct UserFlag {
    pub user_id: u64,
    pub flag: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

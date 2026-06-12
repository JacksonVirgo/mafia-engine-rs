use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct Signup {
    pub message_id: u64,
    pub name: String,
    pub is_anonymous: bool,
    pub created_at: DateTime<Utc>,
}

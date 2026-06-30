use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct VoteCounterSlot {
    pub id: u64,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

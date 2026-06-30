use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct VoteCounterSlotMember {
    pub slot_id: u64,
    pub member_id: u64,
    pub created_at: DateTime<Utc>,
}

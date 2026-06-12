use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct SignupMember {
    pub id: u64,
    pub slot_id: u64,
    pub member_id: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

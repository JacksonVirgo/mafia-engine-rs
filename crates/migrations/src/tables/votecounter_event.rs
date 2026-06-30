use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct VoteCounterEvent {
    pub id: u64,
    pub channel_id: u64,
    pub slot_id: Option<u64>,
    pub vote_target: Option<u64>,
    pub is_skipping: Option<bool>,
    pub is_unvoting: Option<bool>,
    pub vote_weight: Option<i32>,
    pub vote_penalty: Option<i32>,
    pub is_dead: Option<bool>,
    pub can_be_voted: Option<bool>,
    pub can_vote: Option<bool>,
    pub counts_for_majority: Option<bool>,
    pub timed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

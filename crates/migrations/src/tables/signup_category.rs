use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct SignupCategory {
    pub id: u64,
    pub name: String,
    pub button_name: Option<String>,
    pub member_limit: Option<u32>,
    pub is_hoisted: bool,
    pub hydras_permitted: bool,
    pub hydras_forced: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct SignupUser {
    pub id: u64,
    pub slot_id: u64,
    pub user_id: u64,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

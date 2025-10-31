use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct SignupSlot {
    pub id: u64,
    pub category_id: u64,

    pub name: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

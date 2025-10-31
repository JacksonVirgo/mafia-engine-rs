use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};

use crate::data::SqlBool;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct SignupCategory {
    pub id: u64,
    pub signup_id: u64,

    pub name: String,
    pub button_name: String,
    pub max_slots: Option<u8>,
    pub is_hoisted: SqlBool,
    pub is_anonymous: SqlBool,
    pub allows_hydra_slots: SqlBool,
    pub allows_single_slots: SqlBool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

use anyhow::Context;
use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

use crate::Database;

#[derive(Debug, Clone, FromRow)]
pub struct SignupSlot {
    pub id: u64,
    pub category_id: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SignupSlot {
    pub async fn fetch_many(db: &Database, category_id: u64) -> anyhow::Result<Vec<SignupSlot>> {
        sqlx::query_as!(
            SignupSlot,
            "SELECT * FROM signup_slots WHERE category_id = ?",
            category_id,
        )
        .fetch_all(db)
        .await
        .with_context(|| format!("failed to fetch signup slots for category {category_id}"))
    }
}

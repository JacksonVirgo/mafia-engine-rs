use anyhow::Context;
use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

use crate::Database;

#[derive(Debug, Clone, FromRow)]
pub struct SignupCategory {
    pub id: u64,
    pub signup_id: u64,
    pub name: String,
    pub button_name: Option<String>,
    pub member_limit: Option<u32>,
    pub is_hoisted: bool,
    pub is_anonymous: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default, Debug, Clone)]
pub struct CategoryBuilder {
    pub(crate) name: String,
    pub(crate) button_name: Option<String>,
    pub(crate) member_limit: Option<u32>,
    pub(crate) is_hoisted: bool,
    pub(crate) is_anonymous: bool,
}

impl CategoryBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn button_name(mut self, name: impl Into<String>) -> Self {
        self.button_name = Some(name.into());
        self
    }

    pub fn max_slots(mut self, limit: u32) -> Self {
        self.member_limit = Some(limit);
        self
    }

    pub fn set_hoisted(mut self, val: bool) -> Self {
        self.is_hoisted = val;
        self
    }

    pub fn set_anonymous(mut self, val: bool) -> Self {
        self.is_anonymous = val;
        self
    }
}

impl SignupCategory {
    pub async fn fetch_many(db: &Database, signup_id: u64) -> anyhow::Result<Vec<SignupCategory>> {
        sqlx::query_as!(
            SignupCategory,
            r#"SELECT
                id,
                signup_id,
                name,
                button_name,
                member_limit,
                is_hoisted as `is_hoisted: bool`,
                is_anonymous as `is_anonymous: bool`,
                created_at,
                updated_at
               FROM signup_categories
               WHERE signup_id = ?"#,
            signup_id,
        )
        .fetch_all(db)
        .await
        .with_context(|| format!("failed to fetch signup categories for signup {signup_id}"))
    }
}

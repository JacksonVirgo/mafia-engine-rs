use anyhow::Context;
use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

use crate::Database;
use crate::tables::signup_category::CategoryBuilder;

#[derive(Debug, Clone, FromRow)]
pub struct Signup {
    pub message_id: u64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SignupRoster {
    pub category_id: u64,
    pub category_name: String,
    pub button_name: Option<String>,
    pub member_limit: Option<u32>,
    pub is_hoisted: bool,
    pub is_anonymous: bool,
    pub members: Vec<RosterMember>,
}

#[derive(Debug, Clone)]
pub struct RosterMember {
    pub member_id: u64,
    pub username: String,
}

pub struct SignupBuilder {
    message_id: u64,
    name: String,
    categories: Vec<CategoryBuilder>,
}

impl SignupBuilder {
    pub fn new(message_id: u64, name: impl Into<String>) -> Self {
        Self {
            message_id,
            name: name.into(),
            categories: Vec::new(),
        }
    }

    pub fn add_categories(mut self, categories: Vec<CategoryBuilder>) -> Self {
        self.categories.extend(categories);
        self
    }

    pub async fn insert_in_db(self, db: &Database) -> anyhow::Result<()> {
        let mut tx = db.begin().await?;

        sqlx::query!(
            "INSERT INTO signups (message_id, name) VALUES (?, ?)",
            self.message_id,
            self.name,
        )
        .execute(&mut *tx)
        .await
        .with_context(|| format!("failed to insert signup {}", self.message_id))?;

        for cat in self.categories {
            sqlx::query!(
                "INSERT INTO signup_categories \
                 (signup_id, name, button_name, member_limit, is_hoisted, is_anonymous) \
                 VALUES (?, ?, ?, ?, ?, ?)",
                self.message_id,
                cat.name,
                cat.button_name,
                cat.member_limit,
                cat.is_hoisted,
                cat.is_anonymous,
            )
            .execute(&mut *tx)
            .await
            .with_context(|| format!("failed to insert category '{}'", cat.name))?;
        }

        tx.commit().await.context("failed to commit signup insert")?;
        Ok(())
    }
}

impl Signup {
    pub async fn rename(db: &Database, message_id: u64, new_name: &str) -> anyhow::Result<()> {
        let name = new_name.trim();
        if name.is_empty() {
            anyhow::bail!("signup name cannot be empty");
        }
        let name = name.chars().take(32).collect::<String>();
        sqlx::query!(
            "UPDATE signups SET name = ? WHERE message_id = ?",
            name,
            message_id,
        )
        .execute(db)
        .await
        .with_context(|| format!("failed to rename signup {message_id}"))?;
        Ok(())
    }

    pub async fn fetch_one(db: &Database, message_id: u64) -> anyhow::Result<Signup> {
        sqlx::query_as!(
            Signup,
            "SELECT message_id, name, created_at, updated_at FROM signups WHERE message_id = ?",
            message_id,
        )
        .fetch_one(db)
        .await
        .with_context(|| format!("failed to fetch signup with message_id {message_id}"))
    }

    pub async fn fetch_roster(db: &Database, message_id: u64) -> anyhow::Result<Vec<SignupRoster>> {
        let rows = sqlx::query!(
            r#"SELECT
                sc.id           as `category_id: u64`,
                sc.name         as category_name,
                sc.button_name,
                sc.member_limit,
                sc.is_hoisted   as `is_hoisted: bool`,
                sc.is_anonymous as `is_anonymous: bool`,
                sm.member_id    as `member_id?: u64`,
                u.username      as `username?: String`
               FROM signup_categories sc
               LEFT JOIN signup_slots   ss ON ss.category_id = sc.id
               LEFT JOIN signup_members sm ON sm.slot_id    = ss.id
               LEFT JOIN users          u  ON u.user_id     = sm.member_id
               WHERE sc.signup_id = ?
               ORDER BY sc.id, ss.id, sm.id"#,
            message_id,
        )
        .fetch_all(db)
        .await
        .with_context(|| format!("failed to fetch roster for signup {message_id}"))?;

        let mut roster: Vec<SignupRoster> = Vec::new();
        for row in rows {
            let category_id = row.category_id;
            let entry = match roster.last_mut() {
                Some(last) if last.category_id == category_id => last,
                _ => {
                    roster.push(SignupRoster {
                        category_id,
                        category_name: row.category_name,
                        button_name: row.button_name,
                        member_limit: row.member_limit,
                        is_hoisted: row.is_hoisted,
                        is_anonymous: row.is_anonymous,
                        members: Vec::new(),
                    });
                    roster.last_mut().unwrap()
                }
            };
            if let (Some(member_id), Some(username)) = (row.member_id, row.username) {
                entry.members.push(RosterMember {
                    member_id,
                    username,
                });
            }
        }
        Ok(roster)
    }
}

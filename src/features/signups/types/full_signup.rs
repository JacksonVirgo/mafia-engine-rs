use crate::prelude::*;

pub struct FullSignup {
    pub signup: Signup,
    pub categories: Vec<SignupCategory>,
    pub slots: Vec<SignupSlot>,
    pub users: Vec<SignupUser>,
}

impl FullSignup {
    pub async fn fetch(db: &Database, message_id: u64) -> anyhow::Result<FullSignup> {
        let mut tx = db.begin().await?;

        let signup = sqlx::query_as!(
            Signup,
            "SELECT * FROM signups WHERE message_id = ?",
            message_id
        )
        .fetch_one(&mut *tx)
        .await?;

        let categories = sqlx::query_as!(
            SignupCategory,
            "SELECT * FROM signup_categories WHERE signup_id = ?",
            signup.message_id
        )
        .fetch_all(&mut *tx)
        .await?;

        let slots = sqlx::query_as!(
            SignupSlot,
            "SELECT * FROM signup_slots WHERE category_id IN (SELECT id FROM signup_categories WHERE signup_id = ?)",
            signup.message_id
        )
        .fetch_all(&mut *tx)
        .await?;

        let users = sqlx::query_as!(
            SignupUser,
            "SELECT * FROM signup_users WHERE slot_id IN (SELECT id FROM signup_slots WHERE category_id IN (SELECT id FROM signup_categories WHERE signup_id = ?))",
            signup.message_id
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(FullSignup {
            signup,
            categories: categories,
            slots: slots,
            users: users,
        })
    }
}

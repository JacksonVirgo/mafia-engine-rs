use anyhow::Context;
use sqlx::{
    FromRow,
    types::chrono::{DateTime, Utc},
};

use crate::Database;

#[derive(Debug, Clone, FromRow)]
pub struct SignupMember {
    pub id: u64,
    pub slot_id: u64,
    pub member_id: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinResult {
    Joined,
    AlreadyJoined,
    Full,
    UnknownCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminAddResult {
    Added,
    AlreadyInCategory,
    Full,
    UnknownCategory,
}

impl SignupMember {
    pub async fn fetch_many(db: &Database, slot_id: u64) -> anyhow::Result<Vec<SignupMember>> {
        sqlx::query_as!(
            SignupMember,
            "SELECT * FROM signup_members WHERE slot_id = ?",
            slot_id,
        )
        .fetch_all(db)
        .await
        .with_context(|| format!("failed to fetch signup members for slot {slot_id}"))
    }

    pub async fn join(
        db: &Database,
        category_id: u64,
        user_id: u64,
        username: &str,
    ) -> anyhow::Result<JoinResult> {
        let username = username.chars().take(32).collect::<String>();
        let mut tx = db.begin().await?;

        sqlx::query!(
            "INSERT IGNORE INTO users (user_id, username) VALUES (?, ?)",
            user_id,
            username,
        )
        .execute(&mut *tx)
        .await
        .context("failed to ensure user exists")?;

        let Some(category) = sqlx::query!(
            "SELECT signup_id, member_limit FROM signup_categories WHERE id = ?",
            category_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .context("failed to load category")?
        else {
            return Ok(JoinResult::UnknownCategory);
        };

        let already = sqlx::query!(
            "SELECT sm.id FROM signup_members sm \
             JOIN signup_slots ss ON sm.slot_id = ss.id \
             WHERE ss.category_id = ? AND sm.member_id = ? LIMIT 1",
            category_id,
            user_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .context("failed to check membership")?;

        if already.is_some() {
            return Ok(JoinResult::AlreadyJoined);
        }

        if let Some(limit) = category.member_limit {
            let count = sqlx::query!(
                "SELECT COUNT(*) as count FROM signup_members sm \
                 JOIN signup_slots ss ON sm.slot_id = ss.id \
                 WHERE ss.category_id = ?",
                category_id,
            )
            .fetch_one(&mut *tx)
            .await
            .context("failed to count category members")?;
            if count.count >= limit as i64 {
                return Ok(JoinResult::Full);
            }
        }

        sqlx::query!(
            "DELETE ss FROM signup_slots ss \
             JOIN signup_categories sc ON ss.category_id = sc.id \
             JOIN signup_members sm ON sm.slot_id = ss.id \
             WHERE sc.signup_id = ? AND sc.is_hoisted = 0 AND sm.member_id = ?",
            category.signup_id,
            user_id,
        )
        .execute(&mut *tx)
        .await
        .context("failed to clear other unhoisted memberships")?;

        let slot = sqlx::query!(
            "INSERT INTO signup_slots (category_id) VALUES (?)",
            category_id,
        )
        .execute(&mut *tx)
        .await
        .context("failed to insert slot")?;

        sqlx::query!(
            "INSERT INTO signup_members (slot_id, member_id) VALUES (?, ?)",
            slot.last_insert_id(),
            user_id,
        )
        .execute(&mut *tx)
        .await
        .context("failed to insert member")?;

        tx.commit().await.context("failed to commit join")?;
        Ok(JoinResult::Joined)
    }

    pub async fn admin_add(
        db: &Database,
        category_id: u64,
        user_id: u64,
        username: &str,
    ) -> anyhow::Result<AdminAddResult> {
        let username = username.chars().take(32).collect::<String>();
        let mut tx = db.begin().await?;

        sqlx::query!(
            "INSERT IGNORE INTO users (user_id, username) VALUES (?, ?)",
            user_id,
            username,
        )
        .execute(&mut *tx)
        .await
        .context("failed to ensure user exists")?;

        let Some(category) = sqlx::query!(
            "SELECT member_limit FROM signup_categories WHERE id = ?",
            category_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .context("failed to load category")?
        else {
            return Ok(AdminAddResult::UnknownCategory);
        };

        let already = sqlx::query!(
            "SELECT sm.id FROM signup_members sm \
             JOIN signup_slots ss ON sm.slot_id = ss.id \
             WHERE ss.category_id = ? AND sm.member_id = ? LIMIT 1",
            category_id,
            user_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .context("failed to check membership")?;

        if already.is_some() {
            return Ok(AdminAddResult::AlreadyInCategory);
        }

        if let Some(limit) = category.member_limit {
            let count = sqlx::query!(
                "SELECT COUNT(*) as count FROM signup_members sm \
                 JOIN signup_slots ss ON sm.slot_id = ss.id \
                 WHERE ss.category_id = ?",
                category_id,
            )
            .fetch_one(&mut *tx)
            .await
            .context("failed to count category members")?;
            if count.count >= limit as i64 {
                return Ok(AdminAddResult::Full);
            }
        }

        let slot = sqlx::query!(
            "INSERT INTO signup_slots (category_id) VALUES (?)",
            category_id,
        )
        .execute(&mut *tx)
        .await
        .context("failed to insert slot")?;

        sqlx::query!(
            "INSERT INTO signup_members (slot_id, member_id) VALUES (?, ?)",
            slot.last_insert_id(),
            user_id,
        )
        .execute(&mut *tx)
        .await
        .context("failed to insert member")?;

        tx.commit().await.context("failed to commit admin_add")?;
        Ok(AdminAddResult::Added)
    }

    pub async fn admin_remove(
        db: &Database,
        category_id: u64,
        user_id: u64,
    ) -> anyhow::Result<u64> {
        let result = sqlx::query!(
            "DELETE ss FROM signup_slots ss \
             JOIN signup_members sm ON sm.slot_id = ss.id \
             WHERE ss.category_id = ? AND sm.member_id = ?",
            category_id,
            user_id,
        )
        .execute(db)
        .await
        .with_context(|| {
            format!("failed to remove user {user_id} from category {category_id}")
        })?;
        Ok(result.rows_affected())
    }

    pub async fn leave_signup(
        db: &Database,
        signup_id: u64,
        user_id: u64,
    ) -> anyhow::Result<u64> {
        let result = sqlx::query!(
            "DELETE ss FROM signup_slots ss \
             JOIN signup_categories sc ON ss.category_id = sc.id \
             JOIN signup_members sm ON sm.slot_id = ss.id \
             WHERE sc.signup_id = ? AND sc.is_hoisted = 0 AND sm.member_id = ?",
            signup_id,
            user_id,
        )
        .execute(db)
        .await
        .with_context(|| format!("failed to leave signup {signup_id} for user {user_id}"))?;
        Ok(result.rows_affected())
    }
}

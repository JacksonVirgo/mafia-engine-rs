use crate::prelude::{Database, SignupCategory, SignupSlot};

pub async fn user_join_signup(db: &Database, category_id: u64, user_id: u64) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;

    let category = sqlx::query_as!(
        SignupCategory,
        "SELECT * FROM signup_categories WHERE id = ? FOR UPDATE;",
        category_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // No lock here - if race conditions still occur. Add a lock here (and delete this comment)
    let existing_slots = sqlx::query_as!(
        SignupSlot,
        "SELECT * FROM signup_slots WHERE category_id = ?",
        category_id
    )
    .fetch_all(&mut *tx)
    .await?;

    match category.max_slots {
        None => {}
        Some(limit) => {
            if existing_slots.len() >= limit as usize {
                tx.rollback().await?;
                return Err(anyhow::anyhow!("Category full"));
            }
        }
    }

    let is_user_existing = sqlx::query_scalar!(
        r#"
        SELECT 1
        FROM signup_users u
        JOIN signup_slots s ON u.slot_id = s.id
        WHERE s.category_id = ? AND u.user_id = ?
        LIMIT 1
        "#,
        category_id,
        user_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    if is_user_existing.is_some() {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("User already exists"));
    }

    sqlx::query!(
        "INSERT INTO signup_slots (category_id) VALUES (?)",
        category_id
    )
    .execute(&mut *tx)
    .await?;

    let slot_id: u64 = sqlx::query_scalar!("SELECT LAST_INSERT_ID()")
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query!(
        "INSERT INTO signup_users (slot_id, user_id) VALUES (?, ?)",
        slot_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

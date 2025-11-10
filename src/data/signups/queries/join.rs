use crate::prelude::{Database, SignupCategory, SignupSlot};

#[derive(Debug, Clone)]
pub enum UserJoinSignupError {
    CategoryFull,
    UserAlreadyExists,
    NotFound(String),
    Other(String, Option<String>),
}

pub async fn user_join_signup(
    db: &Database,
    category_id: u64,
    user_id: u64,
) -> Result<(), UserJoinSignupError> {
    let Ok(mut tx) = db.begin().await else {
        return Err(UserJoinSignupError::Other(
            "Failed to start DB transaction".into(),
            None,
        ));
    };

    let Ok(category) = sqlx::query_as!(
        SignupCategory,
        "SELECT * FROM signup_categories WHERE id = ? FOR UPDATE;",
        category_id
    )
    .fetch_one(&mut *tx)
    .await
    else {
        return Err(UserJoinSignupError::NotFound(format!(
            "Category `{}` not found.",
            category_id
        )));
    };

    // No lock here - if race conditions still occur. Add a lock here (and delete this comment)
    let existing_slots = match sqlx::query_as!(
        SignupSlot,
        "SELECT * FROM signup_slots WHERE category_id = ?",
        category_id
    )
    .fetch_all(&mut *tx)
    .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                format!(
                    "Failed to fetch existing slots in category `{}`",
                    category_id
                ),
                Some(e.to_string()),
            ));
        }
    };

    match category.max_slots {
        None => {}
        Some(limit) => {
            if existing_slots.len() >= limit as usize {
                let _ = tx.rollback().await;
                return Err(UserJoinSignupError::CategoryFull);
            }
        }
    }

    let is_user_existing = match sqlx::query_scalar!(
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
    .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                format!(
                    "Failed to check if user already is in category `{}`",
                    category_id
                ),
                Some(e.to_string()),
            ));
        }
    };

    if is_user_existing.is_some() {
        let _ = tx.rollback().await;
        return Err(UserJoinSignupError::UserAlreadyExists);
    }

    match sqlx::query!(
        "INSERT INTO signup_slots (category_id) VALUES (?)",
        category_id
    )
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                "Failed to insert new slot".into(),
                Some(e.to_string()),
            ));
        }
    };

    let slot_id: u64 = match sqlx::query_scalar!("SELECT LAST_INSERT_ID()")
        .fetch_one(&mut *tx)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                "Failed to get inserted slot ID".into(),
                Some(e.to_string()),
            ));
        }
    };

    match sqlx::query!(
        "INSERT INTO signup_users (slot_id, user_id) VALUES (?, ?)",
        slot_id,
        user_id
    )
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                "Failed to insert user into slot".into(),
                Some(e.to_string()),
            ));
        }
    }

    match sqlx::query!(
        r#"
          DELETE u FROM signup_users u
          JOIN signup_slots s ON u.slot_id = s.id
          JOIN signup_categories c ON s.category_id = c.id
          WHERE c.signup_id = ? AND u.user_id = ? AND c.is_hoisted = FALSE AND s.id != ?
          "#,
        category.signup_id,
        user_id,
        slot_id
    )
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                "Failed to delete user from old categories".into(),
                Some(e.to_string()),
            ));
        }
    };

    match sqlx::query!(
        r#"
        DELETE s FROM signup_slots s
        LEFT JOIN signup_users u ON s.id = u.slot_id
        WHERE s.category_id = ? AND u.id IS NULL
        "#,
        category_id
    )
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                "Failed to delete stale slots".into(),
                Some(e.to_string()),
            ));
        }
    }

    match tx.commit().await {
        Ok(_) => {}
        Err(e) => {
            return Err(UserJoinSignupError::Other(
                "Failed to commit transaction".into(),
                Some(e.to_string()),
            ));
        }
    }

    Ok(())
}

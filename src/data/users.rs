use crate::app::database::Database;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct User {
    pub id: u64,
}

impl User {
    pub async fn fetch_one(db: &Database, id: u64) -> Option<User> {
        let res = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
            .fetch_one(db)
            .await;

        let Ok(user) = res else {
            return None;
        };

        Some(user)
    }

    pub async fn insert_one(db: &Database, user: User) -> anyhow::Result<User> {
        sqlx::query!("INSERT INTO users (id) VALUES (?)", user.id)
            .execute(db)
            .await?;

        let Some(res) = User::fetch_one(db, user.id).await else {
            return Err(anyhow!("Could not insert/fetch inserted user row"));
        };

        Ok(res)
    }

    pub async fn fetch_or_insert_one(pool: &Database, id: u64) -> anyhow::Result<User> {
        if let Some(fetched) = User::fetch_one(pool, id).await {
            return Ok(fetched);
        };
        let inserted = User::insert_one(pool, User { id }).await?;
        Ok(inserted)
    }
}

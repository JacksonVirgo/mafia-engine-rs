use crate::prelude::*;
use async_trait::async_trait;

pub struct RequireServer;

#[async_trait]
impl Middleware for RequireServer {
    async fn call(&self, req: &mut Request, next: Next<'_>) -> Result<Outcome, BotError> {
        let Some(guild_id) = req.guild_id else {
            return Ok(Outcome::Reject(Rejection::new(
                "This action must be done in a server",
            )));
        };

        let server_id = guild_id.get();
        let db = req.data.db.as_ref();

        sqlx::query!(
            "INSERT IGNORE INTO servers (server_id) VALUES (?)",
            server_id,
        )
        .execute(db)
        .await?;

        let server = sqlx::query_as!(
            db::Server,
            "SELECT server_id, created_at FROM servers WHERE server_id = ?",
            server_id,
        )
        .fetch_one(db)
        .await?;

        req.ext.insert(server);

        next.run(req).await
    }
}

use crate::{
    features::middleware::require_server::RequireServer,
    prelude::{db::Server, *},
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct RequireMember;

#[async_trait]
impl Middleware for RequireMember {
    fn requires(&self) -> Vec<DynMiddleware> {
        vec![Arc::new(RequireServer)]
    }

    async fn call(&self, req: &mut Request, next: Next<'_>) -> Result<Outcome, BotError> {
        let Some(guild_id) = req.guild_id else {
            return Ok(Outcome::Reject(Rejection::new(
                "This action must be done in a server",
            )));
        };
        let server = req.ext.get::<Server>()?;
        let server_id = server.server_id;
        let user_id = req.user_id.get();
        let db = req.data.db.as_ref();

        let discord_user = req.user_id.to_user(&req.serenity_ctx).await?;
        let discord_member = guild_id.member(&req.serenity_ctx, req.user_id).await?;
        let nickname = discord_member.nick.as_deref();

        sqlx::query!(
            "INSERT IGNORE INTO users (user_id, username) VALUES (?, ?)",
            user_id,
            discord_user.name,
        )
        .execute(db)
        .await?;

        sqlx::query!(
            "INSERT INTO members (user_id, server_id, username) VALUES (?, ?, ?) \
             ON DUPLICATE KEY UPDATE username = VALUES(username)",
            user_id,
            server_id,
            nickname,
        )
        .execute(db)
        .await?;

        let user = sqlx::query_as!(
            db::User,
            "SELECT user_id, username, created_at, updated_at FROM users WHERE user_id = ?",
            user_id,
        )
        .fetch_one(db)
        .await?;

        let member = sqlx::query_as!(
            db::Member,
            "SELECT user_id, server_id, username, created_at, updated_at \
             FROM members WHERE user_id = ? AND server_id = ?",
            user_id,
            server_id,
        )
        .fetch_one(db)
        .await?;

        req.ext.insert(user);
        req.ext.insert(member);

        next.run(req).await
    }
}

use crate::prelude::*;
use async_trait::async_trait;

pub struct RequireManageChannels;

#[async_trait]
impl Middleware for RequireManageChannels {
    async fn call(&self, req: &mut Request, next: Next<'_>) -> Result<Outcome, BotError> {
        let Some(guild_id) = req.guild_id else {
            return Ok(Outcome::Reject(Rejection::new(
                "This action must be done in a server",
            )));
        };

        let member = guild_id.member(&req.serenity_ctx, req.user_id).await?;
        #[allow(deprecated)]
        let permissions = member.permissions(&req.serenity_ctx)?;

        if permissions.manage_channels() {
            next.run(req).await
        } else {
            Ok(Outcome::Reject(Rejection::new(
                "You do not have permission to manage this signup.",
            )))
        }
    }
}

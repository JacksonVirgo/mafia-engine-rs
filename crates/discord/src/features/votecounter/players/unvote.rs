use crate::prelude::*;

#[poise::command(slash_command)]
pub async fn unvote(ctx: BotCtx<'_>) -> Result<(), BotError> {
    ctx.send(poise::CreateReply::default().content("TBD").ephemeral(true))
        .await?;
    Ok(())
}

use crate::prelude::*;

pub mod form;

#[poise::command(slash_command, subcommands("votecount"), subcommand_required)]
pub async fn manage(_: BotCtx<'_>) -> Result<(), BotError> {
    Ok(())
}

/// Manage the votecounter in this channel.
#[poise::command(slash_command)]
pub async fn votecount(ctx: BotCtx<'_>) -> Result<(), BotError> {
    ctx.send(
        poise::CreateReply::default()
            .content("Manage votecount TBD")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

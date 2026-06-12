use crate::prelude::*;

#[poise::command(slash_command)]
pub async fn create(
    ctx: BotCtx<'_>,
    #[description = "Signup name (max 32 chars)"]
    #[max_length = 32]
    name: String,
    #[description = "Don't show who is currently signed up"] anonymous: Option<bool>,
) -> Result<(), BotError> {
    let name = name.trim();
    if name.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content("Signup name cannot be empty.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let anonymous = anonymous.unwrap_or(false);

    ctx.send(
        poise::CreateReply::default()
            .content(format!("Signup **{name}** (anonymous: {anonymous}).",))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

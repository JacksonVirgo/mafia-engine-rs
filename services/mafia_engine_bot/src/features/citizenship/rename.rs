use mafia_discord_framework::prelude::*;

#[slash_command(description = "Rename yourself in the bot")]
pub async fn rename(
    ctx: CommandContext,
    #[description = "new name"] username: String,
) -> Result<(), BoxError> {
    let Some(user) = &ctx.interaction().user else {
        return Err("Interaction does not include user data".into());
    };

    let account_username = &user.name;

    tracing::info!("renaming {account_username} to {username}");

    ctx.respond("Unimplemented").await?;
    Ok(())
}

use crate::{
    app::logging::{LogType, log},
    prelude::*,
};

plugin!(CitizenshipPlugin, |app| {
    app.add_command(view_citizenship());
});

/// See a members citizenship card.
#[poise::command(slash_command, rename = "citizenship", guild_only)]
pub async fn view_citizenship(
    ctx: Context<'_>,
    #[description = "Who you want to see"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.unwrap_or(ctx.author().clone());
    let _ = ctx.say(format!("Hey, {}", user.name)).await;

    log(
        LogType::Critical,
        "AHHHHHHHHHHHHH THE WORLD IS FUCKKKKKEEDDDD",
        None::<String>,
    );

    Ok(())
}

use crate::app::database::{Database, is_unique_violation};
use mafia_discord_framework::prelude::*;

pub fn command(database: Database) -> SlashCommand {
    SlashCommand::new("rename", "Rename yourself in the bot")
        .required_string("username", "new name")
        .handler(move |ctx| {
            let database = database.clone();

            async move {
                let username = ctx.required_string("username")?.to_owned();
                rename(ctx, database, username).await
            }
        })
}

async fn rename(ctx: CommandContext, database: Database, username: String) -> Result<(), BoxError> {
    let username = username.trim();
    if username.is_empty() || username.chars().count() > 32 {
        ctx.respond("Your username must contain between 1 and 32 characters.")
            .await?;
        return Ok(());
    }

    let Some(guild_id) = ctx.interaction().guild_id else {
        ctx.respond("This command can only be used in a server.")
            .await?;
        return Ok(());
    };
    let Some(user) = ctx.interaction().author() else {
        return Err("Interaction does not include user data".into());
    };
    if user.bot {
        return Ok(());
    }

    let result = sqlx::query(
        "UPDATE members
         SET username = ?
         WHERE user_id = ? AND guild_id = ? AND deleted_at IS NULL",
    )
    .bind(username)
    .bind(user.id.get())
    .bind(guild_id.get())
    .execute(&database)
    .await;

    match result {
        Ok(_) => {
            tracing::info!(
                user_id = user.id.get(),
                guild_id = guild_id.get(),
                "member renamed"
            );
            ctx.respond("Your username has been updated.").await?;
        }
        Err(error) if is_unique_violation(&error) => {
            ctx.respond("That username is already taken in this server.")
                .await?;
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

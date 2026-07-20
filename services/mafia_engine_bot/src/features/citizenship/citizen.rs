use crate::app::database::Database;
use mafia_discord_framework::prelude::*;
use twilight_model::{
    channel::message::embed::{Embed, EmbedField, EmbedThumbnail},
    user::User,
};

pub fn command() -> SlashCommand {
    SlashCommand::new("citizenship", "Show your Mafia Engine citizenship profile")
        .handler(|ctx| async move { citizenship(ctx).await })
}

async fn citizenship(ctx: CommandContext) -> Result<(), BoxError> {
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
    let database = ctx
        .global::<Database>()
        .ok_or("Database is not configured in the global context")?;

    let membership = sqlx::query_as::<_, (Option<String>, i64)>(
        "SELECT username, CAST(UNIX_TIMESTAMP(created_at) AS SIGNED)
         FROM members
         WHERE user_id = ? AND guild_id = ? AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(user.id.get())
    .bind(guild_id.get())
    .fetch_optional(database.as_ref())
    .await?;

    let Some((username, joined_at)) = membership else {
        ctx.respond("You do not have an active citizenship profile in this server.")
            .await?;
        return Ok(());
    };

    let embed = Embed {
        author: None,
        color: Some(0x5865F2),
        description: None,
        fields: vec![
            EmbedField {
                inline: true,
                name: "Bot username".into(),
                value: username.unwrap_or_else(|| "Deleted User".into()),
            },
            EmbedField {
                inline: true,
                name: "Joined Mafia Engine".into(),
                value: format!("<t:{joined_at}:D>"),
            },
        ],
        footer: None,
        image: None,
        kind: "rich".into(),
        provider: None,
        thumbnail: Some(EmbedThumbnail {
            height: None,
            proxy_url: None,
            url: avatar_url(user),
            width: None,
        }),
        timestamp: None,
        title: Some(format!("{}'s Citizenship", user.name)),
        url: None,
        video: None,
    };

    ctx.respond_embed(embed).await
}

fn avatar_url(user: &User) -> String {
    if let Some(avatar) = user.avatar {
        let extension = if avatar.is_animated() { "gif" } else { "png" };
        return format!(
            "https://cdn.discordapp.com/avatars/{}/{}.{}?size=256",
            user.id, avatar, extension
        );
    }

    let index = if user.discriminator == 0 {
        (user.id.get() >> 22) % 6
    } else {
        u64::from(user.discriminator % 5)
    };
    format!("https://cdn.discordapp.com/embed/avatars/{index}.png")
}

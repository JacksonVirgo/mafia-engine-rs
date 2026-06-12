use crate::prelude::*;
use poise::serenity_prelude::{CreateAllowedMentions, Permissions};

#[poise::command(slash_command)]
pub async fn rename(
    ctx: BotCtx<'_>,
    #[description = "New username (max 32 chars)"]
    #[max_length = 32]
    name: String,
    #[description = "Admins only: a member to rename instead of yourself"] user: Option<
        serenity::Member,
    >,
) -> Result<(), BotError> {
    let name = name.trim();
    if name.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content("Name cannot be empty.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let server_id = ext::<db::Member>(ctx).await?.server_id;

    let (target_user_id, target_discord_name): (u64, String) = match user {
        Some(target) => {
            let guild_id = ctx
                .guild_id()
                .ok_or_else(|| -> BotError { "Must be used in a server.".into() })?;
            let invoker = guild_id
                .member(ctx.serenity_context(), ctx.author().id)
                .await?;

            let is_admin = match invoker.permissions {
                Some(perms) => perms.contains(Permissions::ADMINISTRATOR),
                None => false,
            };

            if !is_admin {
                ctx.send(
                    poise::CreateReply::default()
                        .content("Only administrators may rename other members.")
                        .ephemeral(true),
                )
                .await?;
                return Ok(());
            }
            (target.user.id.get(), target.user.name.clone())
        }
        None => (ctx.author().id.get(), ctx.author().name.clone()),
    };

    let db = ctx.data().db.as_ref();

    sqlx::query!(
        "INSERT IGNORE INTO users (user_id, username) VALUES (?, ?)",
        target_user_id,
        target_discord_name,
    )
    .execute(db)
    .await?;

    sqlx::query!(
        "INSERT INTO members (user_id, server_id, username) VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE username = VALUES(username)",
        target_user_id,
        server_id,
        name,
    )
    .execute(db)
    .await?;

    ctx.send(
        poise::CreateReply::default()
            .content(format!("Renamed <@{target_user_id}> to **{name}**."))
            .allowed_mentions(CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}

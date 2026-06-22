use crate::features::signups::embed::fetch_and_format_signup;
use crate::prelude::*;
use db::{CategoryBuilder, SignupBuilder};

#[poise::command(slash_command)]
pub async fn create(
    ctx: BotCtx<'_>,
    #[description = "Signup name (max 32 chars)"]
    #[max_length = 32]
    name: String,
    #[description = "Player slot count"] limit: Option<u32>,
    #[description = "Don't show who is currently signed up"] anonymous: Option<bool>,
) -> Result<(), BotError> {
    let name = name.trim().to_string();
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

    let handle = ctx
        .send(poise::CreateReply::default().content("Creating signup..."))
        .await?;
    let message = handle.message().await?;
    let message_id = message.id.get();

    let mut players = CategoryBuilder::default()
        .name("Players")
        .button_name("Play")
        .set_anonymous(anonymous);
    if let Some(limit) = limit {
        players = players.max_slots(limit);
    }

    let builder = SignupBuilder::new(message_id, name.clone()).add_categories(vec![
        CategoryBuilder::default().name("Hosts").set_hoisted(true),
        CategoryBuilder::default()
            .name("Moderators")
            .set_hoisted(true),
        CategoryBuilder::default()
            .name("Balancers")
            .set_hoisted(true),
        players,
        CategoryBuilder::default()
            .name("Backups")
            .button_name("Backup")
            .set_anonymous(anonymous),
    ]);

    if let Err(e) = builder.insert_in_db(&ctx.data().db).await {
        error!("Failed to add new signup into database: {e:?}");
        handle
            .edit(
                ctx,
                poise::CreateReply::default().content("Failed to create signup."),
            )
            .await?;
        return Ok(());
    }

    let (embed, components) = fetch_and_format_signup(&ctx.data().db, message_id).await?;
    handle
        .edit(
            ctx,
            poise::CreateReply::default()
                .content("")
                .embed(embed)
                .components(components),
        )
        .await?;

    Ok(())
}

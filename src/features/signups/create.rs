use poise::{CreateReply, serenity_prelude::EditMessage};

use crate::{
    app::logging::log,
    features::signups::{
        dashboard::signup_dashboard,
        types::{
            builder::{CategoryBuilder, SignupBuilder},
            full_signup::FullSignup,
        },
    },
    prelude::*,
};

/// Create a new signup
#[poise::command(slash_command, rename = "create", guild_only)]
pub async fn create_signups(
    ctx: Context<'_>,
    #[description = "Player limit"] limit: Option<u8>,
    #[description = "Signup anonymous?"] anonymous: Option<bool>,
) -> Result<(), Error> {
    let raw_msg = ctx
        .send(CreateReply::default().content("Loading..."))
        .await?;

    let mut message = raw_msg.into_message().await?;

    match SignupBuilder::new(message.id.get())
        .add_categories(vec![
            CategoryBuilder::default().name("Hosts").set_hoisted(true),
            CategoryBuilder::default()
                .name("Moderators")
                .set_hoisted(true),
            CategoryBuilder::default()
                .name("Balancers")
                .set_hoisted(true),
            CategoryBuilder::default()
                .name("Players")
                .button_name("Play")
                .max_slots(limit)
                .set_anonymous(anonymous.unwrap_or(false)),
            CategoryBuilder::default()
                .name("Backups")
                .button_name("Backup")
                .set_anonymous(anonymous.unwrap_or(false)),
        ])
        .insert_in_db(&ctx.data().db)
        .await
    {
        Err(e) => {
            log(
                LogType::Error,
                "Failed to add new signup into database",
                Some(e.to_string()),
            );
        }
        _ => {}
    };

    let full_signup = FullSignup::fetch(&ctx.data().db, message.id.get()).await?;

    let (embed, components) = signup_dashboard(full_signup).await;

    let _ = message
        .edit(
            &ctx.http(),
            EditMessage::default()
                .embed(embed)
                .components(components)
                .content(""),
        )
        .await;

    Ok(())
}

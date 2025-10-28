use std::sync::Arc;

use crate::app::database::Database;
use crate::app::discord::event_handler::event_handler;
use crate::features::testing::command::test;
use poise::serenity_prelude::{self as serenity, GatewayIntents};

pub mod event_handler;

pub struct ContextData {
    pub db: Arc<Database>,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ContextData, Error>;

pub async fn setup_discord(db: Database) -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![test()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            let db = Arc::new(db);
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(ContextData { db })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    Ok(())
}

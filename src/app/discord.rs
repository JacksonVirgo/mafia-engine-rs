use crate::app::event_handler::event_handler;
use crate::app::system::app_builder::AppBuilder;
use crate::app::{database::Database, system::plugin::Plugin};
use crate::features::FeaturePlugin;
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use std::sync::Arc;

#[derive(Clone)]
pub struct ContextData {
    pub db: Arc<Database>,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ContextData, Error>;

pub async fn setup_discord(db: Database) -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let mut commands: Vec<poise::Command<ContextData, Error>> = Vec::new();

    let mut app_builder = AppBuilder {
        commands: &mut commands,
    };

    InitialPlugin.build(&mut app_builder);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands,
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

pub struct InitialPlugin;
impl Plugin for InitialPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FeaturePlugin);
    }
}

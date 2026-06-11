use crate::{
    app::{event_handler::event_handler, system::plugin::Plugin},
    features::FeaturePlugin,
    prelude::AppBuilder,
};
use poise::serenity_prelude::*;

#[derive(Clone)]
pub struct BotState {
    pub started_at: std::time::Instant,
}

pub type BotError = Box<dyn std::error::Error + Send + Sync>;
pub type BotCtx<'a> = poise::Context<'a, BotState, BotError>;

pub async fn setup_discord_bot() -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_CLIENT_SECRET")
        .expect("missing DISCORD_CLIENT_SECRET environment variable");

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let mut commands: Vec<poise::Command<BotState, BotError>> = Vec::new();
    InitialPlugin.build(&mut AppBuilder { commands: &mut commands });

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(BotState { started_at: std::time::Instant::now() })
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, intents)
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

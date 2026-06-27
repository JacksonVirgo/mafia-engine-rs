use bot_framework::prelude::*;

pub mod features;

pub type State = ();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let token = std::env::var("DISCORD_CLIENT_SECRET")
        .map_err(|_| anyhow::anyhow!("DISCORD_CLIENT_SECRET env var not set"))?;

    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::MESSAGE_CONTENT;

    Bot::new(token, intents, ())
        .add_plugin(features::FeaturePlugin)
        .run()
        .await
}

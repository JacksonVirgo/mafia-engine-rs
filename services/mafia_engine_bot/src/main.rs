use mafia_discord_framework::{App, prelude::Intents};
use mafia_engine_bot::{
    app::database::{DatabasePlugin, setup_database},
    features::CorePlugin,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database = setup_database().await?;
    let token = std::env::var("DISCORD_TOKEN")?;
    let mut app = App::new(token, Intents::all());
    app.global_context().insert(database);
    app.add_plugin(DatabasePlugin);
    app.add_plugin(CorePlugin);
    app.run().await?;

    Ok(())
}

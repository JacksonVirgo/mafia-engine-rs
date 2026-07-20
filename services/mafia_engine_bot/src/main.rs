use mafia_discord_framework::{App, prelude::Intents};
use mafia_engine_bot::{
    app::database::{DatabasePlugin, setup_database},
    features::CorePlugin,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database = setup_database().await?;
    let token = std::env::var("DISCORD_TOKEN")?;
    let mut app = App::new(token, Intents::all());
    app.add_plugin(DatabasePlugin::new(database));
    app.add_plugin(CorePlugin);
    let _ = app.run().await;

    Ok(())
}

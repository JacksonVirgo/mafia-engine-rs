use mafia_discord_framework::{App, prelude::Intents};
use mafia_engine_bot::features::CorePlugin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN")?;
    let mut app = App::new(token, Intents::all());
    app.add_plugin(CorePlugin);
    let _ = app.run().await;

    Ok(())
}

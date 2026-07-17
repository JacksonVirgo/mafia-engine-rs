use mafia_discord_framework::prelude::*;

struct CorePlugin;

async fn on_ready(ready: Ready, _context: EventContext) -> Result<(), BoxError> {
    println!("connected as {}", ready.user.name);
    Ok(())
}

#[slash_command(description = "Pong!")]
async fn ping(
    ctx: CommandContext,
    #[description = "Ping?"] message: Option<String>,
) -> Result<(), BoxError> {
    ctx.respond(format!("Pong: {}", message.unwrap_or("None".into())))
        .await
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event_listener(on_ready).add_interaction(ping());
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN")?;
    let mut app = App::new(token, Intents::GUILDS);
    app.add_plugin(CorePlugin);
    app.run().await
}

use discord::app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    app::discord::setup_discord_bot().await?;

    Ok(())
}

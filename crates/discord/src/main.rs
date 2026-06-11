use discord::app;
use migrations::{run_migrations, setup_database};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = setup_database().await?;
    run_migrations(&db).await?;
    app::discord::setup_discord_bot(db).await?;

    Ok(())
}

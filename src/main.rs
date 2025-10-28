use std::i64;

use mafia_engine_rs::app::{database::setup_database, discord::setup_discord};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let _ = tracing_subscriber::fmt::init();

    let db = setup_database().await?;
    setup_discord(db).await;

    Ok(())
}

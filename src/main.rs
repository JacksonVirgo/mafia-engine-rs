use mafia_engine_rs::app::discord::setup_discord;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    setup_discord().await;
}

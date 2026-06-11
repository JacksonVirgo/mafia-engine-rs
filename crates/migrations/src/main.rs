use migrations::{run_migrations, setup_database};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let pool = setup_database().await?;
    run_migrations(&pool).await?;

    Ok(())
}

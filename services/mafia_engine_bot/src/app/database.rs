use anyhow::{Context, Result, bail};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use tokio::time::{Duration, sleep};

pub type Database = MySqlPool;

pub async fn setup_database() -> Result<Database> {
    let connection_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let max_attempts = std::env::var("DATABASE_CONN_MAX_ATTEMPTS").map_or(Ok(10), |value| {
        value
            .parse::<u32>()
            .context("DATABASE_CONN_MAX_ATTEMPTS must be a positive integer")
    })?;

    if max_attempts == 0 {
        bail!("DATABASE_CONN_MAX_ATTEMPTS must be greater than zero");
    }

    let mut last_error = None;
    for attempt in 1..=max_attempts {
        tracing::info!(attempt, max_attempts, "attempting to connect to database");
        match MySqlPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .connect(&connection_url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(error) => {
                tracing::error!(%error, attempt, max_attempts, "database connection failed");
                last_error = Some(error);

                if attempt < max_attempts {
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    Err(
        anyhow::Error::new(last_error.expect("at least one connection attempt was made")).context(
            format!("could not connect to the database after {max_attempts} attempts"),
        ),
    )
}

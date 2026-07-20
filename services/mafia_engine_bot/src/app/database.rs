use anyhow::{Context, Result, anyhow, bail};
use sqlx::migrate::Migrator;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use tokio::time::{Duration, sleep};
use url::Url;

pub type Database = MySqlPool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn setup_database() -> Result<Database> {
    let connection_url = database_url()?;
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
            Ok(pool) => {
                tracing::info!("applying database migrations");
                MIGRATOR
                    .run(&pool)
                    .await
                    .context("failed to apply database migrations")?;
                return Ok(pool);
            }
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

pub fn database_url() -> Result<String> {
    let username = required_env("DATABASE_USER")?;
    let password = required_env("DATABASE_PASSWORD")?;
    let host = required_env("DATABASE_HOST")?;
    let database = required_env("DATABASE_NAME")?;
    let port = required_env("DATABASE_PORT")?
        .parse::<u16>()
        .context("DATABASE_PORT must be a valid port number")?;

    let mut url = Url::parse("mysql://").expect("static MySQL URL is valid");
    url.set_host(Some(&host))
        .context("DATABASE_HOST must be a valid hostname or IP address")?;
    url.set_port(Some(port))
        .map_err(|_| anyhow!("DATABASE_PORT cannot be represented in a database URL"))?;
    url.set_username(&username)
        .map_err(|_| anyhow!("DATABASE_USER cannot be represented in a database URL"))?;
    url.set_password(Some(&password))
        .map_err(|_| anyhow!("DATABASE_PASSWORD cannot be represented in a database URL"))?;
    url.set_path(&database);

    Ok(url.into())
}

fn required_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("{name} must be set"))
}

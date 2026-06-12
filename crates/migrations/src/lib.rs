use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use tokio::time::{Duration, sleep};
use tracing::{error, info};

pub mod flags;
pub mod tables;

pub type Database = Pool<MySql>;

pub async fn setup_database() -> anyhow::Result<Database> {
    let conn = generate_connection_url()?;

    let mut attempts = 0;
    for i in 0..10 {
        info!("Attempting to connect to DB... attempt {}", i + 1);
        match MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&conn)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                attempts += 1;
                error!("DB not ready yet: {}", e);
                sleep(Duration::from_secs(1)).await
            }
        }
    }

    anyhow::bail!("Could not connect to database after {} attempts", attempts);
}

pub async fn run_migrations(pool: &Database) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

fn generate_connection_url() -> anyhow::Result<String> {
    Ok(match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            let user = std::env::var("MYSQL_USER")?;
            let pass = std::env::var("MYSQL_PASSWORD")?;
            let port = std::env::var("MYSQL_PORT")?;
            let name = std::env::var("MYSQL_DATABASE")?;
            format!("mysql://{}:{}@{}:{}/{}", user, pass, "database", port, name)
        }
    })
}

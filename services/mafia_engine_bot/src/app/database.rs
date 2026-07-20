use anyhow::{Context, Result, anyhow, bail};
use mafia_discord_framework::prelude::{App, Event, Plugin};
use sqlx::migrate::Migrator;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlQueryResult};
use tokio::time::{Duration, sleep};
use url::Url;

pub type Database = MySqlPool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub struct DatabasePlugin;

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        let database = app
            .global_context()
            .get::<Database>()
            .expect("DatabasePlugin requires Database in the global context");
        app.add_event_middleware(move |event, _| {
            let database = database.clone();

            async move {
                synchronize_event(database.as_ref(), event.as_ref())
                    .await
                    .map_err(Into::into)
            }
        });
    }
}

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

async fn synchronize_event(database: &Database, event: &Event) -> Result<()> {
    if let Some(guild_id) = event.guild_id() {
        insert_guild(database, guild_id.get()).await?;
    }

    match event {
        Event::Ready(ready) => {
            for guild in &ready.guilds {
                insert_guild(database, guild.id.get()).await?;
            }

            insert_user(database, ready.user.id.get(), ready.user.bot).await
        }
        Event::GuildCreate(guild_create) => {
            let twilight_model::gateway::payload::incoming::GuildCreate::Available(guild) =
                guild_create.as_ref()
            else {
                return Ok(());
            };

            synchronize_members(database, guild.id.get(), &guild.members).await
        }
        Event::UserUpdate(user) => insert_user(database, user.id.get(), user.bot).await,
        Event::MemberAdd(member) => {
            insert_member(
                database,
                member.guild_id.get(),
                member.user.id.get(),
                &member.user.name,
                member.user.bot,
            )
            .await
        }
        Event::MemberUpdate(member) => {
            insert_member(
                database,
                member.guild_id.get(),
                member.user.id.get(),
                &member.user.name,
                member.user.bot,
            )
            .await
        }
        Event::MemberRemove(member) => {
            insert_user(database, member.user.id.get(), member.user.bot).await
        }
        Event::MemberChunk(chunk) => {
            synchronize_members(database, chunk.guild_id.get(), &chunk.members).await
        }
        Event::MessageCreate(message) => {
            synchronize_message(
                database,
                &message.0.author,
                message.0.guild_id.map(|id| id.get()),
            )
            .await
        }
        Event::MessageUpdate(message) => {
            synchronize_message(
                database,
                &message.0.author,
                message.0.guild_id.map(|id| id.get()),
            )
            .await
        }
        Event::InteractionCreate(interaction) => {
            let interaction = &interaction.0;
            let Some(user) = interaction.author() else {
                return Ok(());
            };

            synchronize_message(database, user, interaction.guild_id.map(|id| id.get())).await
        }
        _ => Ok(()),
    }
}

async fn synchronize_message(
    database: &Database,
    user: &twilight_model::user::User,
    guild_id: Option<u64>,
) -> Result<()> {
    if let Some(guild_id) = guild_id {
        insert_member(database, guild_id, user.id.get(), &user.name, user.bot).await
    } else {
        insert_user(database, user.id.get(), user.bot).await
    }
}

async fn synchronize_members(
    database: &Database,
    guild_id: u64,
    members: &[twilight_model::guild::Member],
) -> Result<()> {
    for member in members {
        insert_member(
            database,
            guild_id,
            member.user.id.get(),
            &member.user.name,
            member.user.bot,
        )
        .await?;
    }

    Ok(())
}

async fn insert_guild(database: &Database, guild_id: u64) -> Result<()> {
    ignore_duplicate_key(
        sqlx::query("INSERT INTO guilds (id) VALUES (?)")
            .bind(guild_id)
            .execute(database)
            .await,
        "insert guild",
    )
}

async fn insert_user(database: &Database, user_id: u64, is_bot: bool) -> Result<()> {
    if is_bot {
        return Ok(());
    }

    ignore_duplicate_key(
        sqlx::query("INSERT INTO users (id) VALUES (?)")
            .bind(user_id)
            .execute(database)
            .await,
        "insert user",
    )
}

async fn insert_member(
    database: &Database,
    guild_id: u64,
    user_id: u64,
    username: &str,
    is_bot: bool,
) -> Result<()> {
    if is_bot {
        return Ok(());
    }

    insert_user(database, user_id, false).await?;

    ignore_duplicate_key(
        sqlx::query("INSERT INTO members (user_id, guild_id, username) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(guild_id)
            .bind(username)
            .execute(database)
            .await,
        "insert member",
    )
}

fn ignore_duplicate_key(
    result: std::result::Result<MySqlQueryResult, sqlx::Error>,
    operation: &str,
) -> Result<()> {
    match result {
        Ok(_) => Ok(()),
        Err(error) if is_unique_violation(&error) => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to {operation}")),
    }
}

pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|database_error| database_error.is_unique_violation())
}

use crate::{
    app::{event_handler::event_handler, system::plugin::Plugin},
    features::FeaturePlugin,
    prelude::*,
};
use migrations::Database;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use std::sync::Arc;

#[derive(Clone)]
pub struct BotState {
    pub started_at: std::time::Instant,
    pub db: Arc<Database>,
}

pub type BotError = Box<dyn std::error::Error + Send + Sync>;
pub type BotCtx<'a> = poise::Context<'a, BotState, BotError>;

pub async fn setup_discord_bot(db: Database) -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_CLIENT_SECRET")
        .expect("missing DISCORD_CLIENT_SECRET environment variable");

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let mut commands: Vec<poise::Command<BotState, BotError>> = Vec::new();
    InitialPlugin.build(&mut AppBuilder {
        commands: &mut commands,
    });

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            command_check: Some(|ctx| Box::pin(command_middleware_check(ctx))),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            let db = Arc::new(db);
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(BotState {
                    started_at: std::time::Instant::now(),
                    db,
                })
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}

async fn command_middleware_check(ctx: BotCtx<'_>) -> Result<bool, BotError> {
    let stack: Vec<DynMiddleware> = ctx
        .command()
        .custom_data
        .downcast_ref::<Vec<DynMiddleware>>()
        .cloned()
        .unwrap_or_default();

    if stack.is_empty() {
        // Nothing to check, but still expose an empty extensions bag so handlers
        // can uniformly call `ext::<T>(ctx)` without first inspecting whether
        // middleware ran.
        ctx.set_invocation_data(Extensions::default()).await;
        return Ok(true);
    }

    let mut req = Request {
        kind: RequestKind::Command {
            name: ctx.command().qualified_name.clone(),
        },
        serenity_ctx: ctx.serenity_context().clone(),
        data: ctx.data().clone(),
        user_id: ctx.author().id,
        guild_id: ctx.guild_id(),
        channel_id: ctx.channel_id(),
        ext: Extensions::default(),
    };

    match crate::app::system::middleware::run_stack(&stack, &mut req).await? {
        Outcome::Continue => {
            ctx.set_invocation_data(req.ext).await;
            Ok(true)
        }
        Outcome::Reject(rej) => {
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content(rej.message)
                        .ephemeral(rej.ephemeral),
                )
                .await;
            Ok(false)
        }
    }
}

pub struct InitialPlugin;
impl Plugin for InitialPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FeaturePlugin);
    }
}

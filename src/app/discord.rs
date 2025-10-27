use poise::serenity_prelude::{
    self as serenity, ButtonStyle, ComponentType, Request, Route, json::json,
};

pub struct ContextData {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ContextData, Error>;

#[poise::command(slash_command)]
async fn test(ctx: Context<'_>) -> Result<(), Error> {
    let (interaction_id, interaction_token) = if let Context::Application(app_ctx) = ctx {
        (app_ctx.interaction.id, app_ctx.interaction.token.as_str())
    } else {
        ctx.say("Not an application interaction").await?;
        return Ok(());
    };

    let body = json!({
        "type": 4,
        "data": {
            "content": "Raw API call",
            "components": [
                {
                    "type": ComponentType::ActionRow,
                    "components": [
                        {
                            "type": ComponentType::Button,
                            "style": ButtonStyle::Primary,
                            "label": "Raw Button",
                            "custom_id": "raw_button_test"
                        }
                    ]
                }
            ]
        }
    });

    let body_str = serde_json::to_string(&body)?;

    let req = Request::new(
        Route::InteractionResponse {
            interaction_id: interaction_id,
            token: interaction_token,
        },
        serenity::LightMethod::Post,
    )
    .body(Some(body_str.into_bytes()));

    ctx.http().request(req).await?;

    Ok(())
}

pub async fn setup_discord() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![test()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(ContextData {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

use crate::{
    app::{
        discord::{Context, Error},
        logging::log,
    },
    data::users::User,
    features::testing::component::TestingButton,
    prelude::{Button, LogType},
};
use poise::serenity_prelude::{self as serenity, ComponentType, Request, Route, json::json};
use tracing::info;

#[poise::command(slash_command)]
pub async fn test(ctx: Context<'_>) -> Result<(), Error> {
    let user = User::fetch_or_insert_one(&ctx.data().db, ctx.author().id.get()).await?;
    info!("User: {:?}", user);

    let (interaction_id, interaction_token) = if let Context::Application(app_ctx) = ctx {
        (app_ctx.interaction.id, app_ctx.interaction.token.as_str())
    } else {
        ctx.say("Not an application interaction").await?;
        return Ok(());
    };

    let button = TestingButton;
    let cmp_button = button.build_as_json().await?;

    let body = json!({
        "type": 4,
        "data": {
            "content": "Raw API call",
            "components": [
                {
                    "type": ComponentType::ActionRow,
                    "components": [
                        cmp_button
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

    log(
        LogType::Info,
        "This should have a file",
        Some("Insert detailed error trace here"),
    );

    Ok(())
}

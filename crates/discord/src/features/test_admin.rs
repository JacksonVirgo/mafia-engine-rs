use crate::prelude::*;
use async_trait::async_trait;
use poise::serenity_prelude::{
    ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

const BUTTON_ID: &str = "test-admin-button";

pub struct RequireAdmin;

#[async_trait]
impl Middleware for RequireAdmin {
    async fn call(&self, req: &mut Request, next: Next<'_>) -> Result<Outcome, BotError> {
        let Some(guild_id) = req.guild_id else {
            return Ok(Outcome::Reject(Rejection::new(
                "This action must be used in a guild.",
            )));
        };

        let member = guild_id.member(&req.serenity_ctx, req.user_id).await?;
        #[allow(deprecated)]
        let permissions = member.permissions(&req.serenity_ctx)?;

        if permissions.administrator() {
            next.run(req).await
        } else {
            Ok(Outcome::Reject(Rejection::new(
                "This action is restricted to administrators.",
            )))
        }
    }
}

#[poise::command(slash_command)]
pub async fn test(ctx: BotCtx<'_>) -> Result<(), BotError> {
    let button = CreateButton::new(BUTTON_ID)
        .label("Admin-only button")
        .style(ButtonStyle::Primary);

    ctx.send(
        poise::CreateReply::default()
            .content("Press the button below — only admins may proceed.")
            .components(vec![CreateActionRow::Buttons(vec![button])]),
    )
    .await?;

    Ok(())
}

pub struct TestAdminButton;

#[async_trait]
impl Component for TestAdminButton {
    fn custom_id(&self) -> String {
        BUTTON_ID.to_string()
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };

        let resp = CreateInteractionResponseMessage::new()
            .content(format!(
                "<@{}> confirmed admin access.",
                component.user.id.get()
            ))
            .ephemeral(true);

        let _ = component
            .create_response(&ctx.ctx.http, CreateInteractionResponse::Message(resp))
            .await;
    }
}

plugin!(TestAdminPlugin, |app| {
    app.add_command(test().with(RequireAdmin));
    app.add_component(TestAdminButton.with(RequireAdmin));
});

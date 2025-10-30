use crate::prelude::*;
use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, CreateButton, CreateInteractionResponseMessage};

pub struct TestingButton;

#[async_trait]
impl Button for TestingButton {
    async fn build(&self) -> CreateButton {
        CreateButton::new("test")
            .label("Test Button Yaya")
            .style(serenity::ButtonStyle::Danger)
    }
}

#[async_trait]
impl Component for TestingButton {
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        if let serenity::Interaction::Component(component) = i {
            // let custom_id = &component.data.custom_id;
            let _ = component
                .create_response(
                    &ctx.ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content("Testy Schmesty"),
                    ),
                )
                .await;
        }
    }
}

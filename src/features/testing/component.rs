use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, CreateInteractionResponseMessage};

use crate::app::component::{Component, ContextBundle};

pub struct TestingButton;

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

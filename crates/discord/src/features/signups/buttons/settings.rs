use crate::{
    app::system::components::{Component, button::Button},
    features::signups::manage::home::render_home,
    prelude::*,
};

use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, CreateButton, ReactionType};

pub struct SignupSettings;

#[async_trait]
impl Button for SignupSettings {
    async fn build(&self) -> CreateButton {
        CreateButton::new(self.custom_id())
            .emoji(ReactionType::Unicode("⚙️".into()))
            .style(serenity::ButtonStyle::Secondary)
    }
}

#[async_trait]
impl Component for SignupSettings {
    fn custom_id(&self) -> String {
        String::from("signup_settings")
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        let signup_id = component.message.id.get();

        let panel = match render_home(&ctx.data.db, signup_id).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to render signup home: {e:?}");
                return;
            }
        };

        let resp = serenity::CreateInteractionResponseMessage::new()
            .embed(panel.embed)
            .components(panel.components)
            .ephemeral(true);
        let _ = component
            .create_response(&ctx.ctx.http, serenity::CreateInteractionResponse::Message(resp))
            .await;
    }
}

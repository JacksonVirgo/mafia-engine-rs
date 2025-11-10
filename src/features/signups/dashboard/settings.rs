use crate::prelude::*;
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

    async fn run(&self, i: &serenity::Interaction, _ctx: ContextBundle) {
        if let serenity::Interaction::Component(_component) = i {
            info!("Settings");
        }
    }
}

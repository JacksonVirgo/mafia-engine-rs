use crate::prelude::*;
use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, CreateButton, ReactionType};

pub struct SignupRefresh;

#[async_trait]
impl Button for SignupRefresh {
    async fn build(&self) -> CreateButton {
        CreateButton::new(self.custom_id())
            .emoji(ReactionType::Unicode("🔄".into()))
            .style(serenity::ButtonStyle::Secondary)
    }
}

#[async_trait]
impl Component for SignupRefresh {
    fn custom_id(&self) -> String {
        String::from("signup_refresh")
    }

    async fn run(&self, i: &serenity::Interaction, _ctx: ContextBundle) {
        if let serenity::Interaction::Component(_component) = i {
            info!("Refresh Query");
        }
    }
}

use crate::{
    app::system::components::{Component, button::Button},
    features::signups::manage::home::update_to_home,
    prelude::*,
};

use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, ButtonStyle, CreateButton, ReactionType};

pub struct SignupPlayerChatsButton;

impl SignupPlayerChatsButton {
    pub fn build_with(&self, signup_id: u64) -> CreateButton {
        CreateButton::new(format!("signup_playerchats:{signup_id}"))
            .emoji(ReactionType::Unicode("👤".into()))
            .style(ButtonStyle::Secondary)
    }
}

#[async_trait]
impl Button for SignupPlayerChatsButton {
    async fn build(&self) -> CreateButton {
        self.build_with(0)
    }
}

#[async_trait]
impl Component for SignupPlayerChatsButton {
    fn custom_id(&self) -> String {
        String::from("signup_playerchats")
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        let Some(signup_id) = ctx.i_ctx.as_deref().and_then(|s| s.parse::<u64>().ok()) else {
            return;
        };
        update_to_home(component, &ctx, signup_id).await;
    }
}

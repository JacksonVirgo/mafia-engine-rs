use crate::{
    app::system::components::{Component, button::Button},
    features::signups::embed::refresh_signup_message,
    prelude::*,
};

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

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        if let Err(e) = refresh_signup_message(component, &ctx.ctx, &ctx.data.db).await {
            error!("Failed to refresh signup embed: {e:?}");
        }
    }
}

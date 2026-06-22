use crate::{
    app::system::components::Component, features::signups::embed::refresh_signup_message,
    prelude::*,
};

use async_trait::async_trait;
use migrations::tables::{JoinResult, SignupMember};
use poise::serenity_prelude::{self as serenity};

pub struct SignupJoin;

#[async_trait]
impl Component for SignupJoin {
    fn custom_id(&self) -> String {
        String::from("signup_join")
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        let Some(category_id) = ctx.i_ctx.as_deref().and_then(|s| s.parse::<u64>().ok()) else {
            error!("signup_join missing/invalid category id");
            return;
        };
        let user_id = component.user.id.get();
        let username = component.user.name.clone();

        match SignupMember::join(&ctx.data.db, category_id, user_id, &username).await {
            Ok(JoinResult::Joined) => {
                if let Err(e) = refresh_signup_message(component, &ctx.ctx, &ctx.data.db).await {
                    error!("Failed to refresh signup after join: {e:?}");
                }
            }
            Ok(JoinResult::AlreadyJoined) => {
                respond_ephemeral(component, &ctx.ctx, "You're already in this category.").await;
            }
            Ok(JoinResult::Full) => {
                respond_ephemeral(component, &ctx.ctx, "This category is full.").await;
            }
            Ok(JoinResult::UnknownCategory) => {
                respond_ephemeral(component, &ctx.ctx, "That category no longer exists.").await;
            }
            Err(e) => {
                error!("Failed to join category {category_id}: {e:?}");
            }
        }
    }
}

async fn respond_ephemeral(
    component: &serenity::ComponentInteraction,
    ctx: &serenity::Context,
    msg: &str,
) {
    let resp = serenity::CreateInteractionResponseMessage::new()
        .content(msg)
        .ephemeral(true);
    let _ = component
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Message(resp))
        .await;
}

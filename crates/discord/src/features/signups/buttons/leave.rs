use crate::{
    app::system::components::{Component, button::Button},
    features::signups::embed::refresh_signup_message,
    prelude::*,
};

use async_trait::async_trait;
use migrations::tables::SignupMember;
use poise::serenity_prelude::{self as serenity, CreateButton};

pub struct SignupLeave;

#[async_trait]
impl Button for SignupLeave {
    async fn build(&self) -> CreateButton {
        CreateButton::new(self.custom_id())
            .emoji('❌')
            .style(serenity::ButtonStyle::Secondary)
    }
}

#[async_trait]
impl Component for SignupLeave {
    fn custom_id(&self) -> String {
        String::from("signup_leave")
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        let signup_id = component.message.id.get();
        let user_id = component.user.id.get();

        match SignupMember::leave_signup(&ctx.data.db, signup_id, user_id).await {
            Ok(0) => {
                let resp = serenity::CreateInteractionResponseMessage::new()
                    .content("You aren't signed up.")
                    .ephemeral(true);
                let _ = component
                    .create_response(
                        &ctx.ctx.http,
                        serenity::CreateInteractionResponse::Message(resp),
                    )
                    .await;
            }
            Ok(_) => {
                if let Err(e) = refresh_signup_message(component, &ctx.ctx, &ctx.data.db).await {
                    error!("Failed to refresh signup after leave: {e:?}");
                }
            }
            Err(e) => {
                error!("Failed to leave signup {signup_id}: {e:?}");
            }
        }
    }
}

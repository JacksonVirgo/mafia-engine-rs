use crate::prelude::*;
use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, ButtonStyle, CreateButton};

#[derive(Default, Clone)]
pub struct JoinSignupBtn {
    pub category: u64,
    pub button_name: String,
    pub is_primary: bool,
    pub is_full: bool,
}

#[async_trait]
impl Button for JoinSignupBtn {
    async fn build(&self) -> CreateButton {
        CreateButton::new(format!("{}:{}", self.custom_id(), self.category))
            .style(if self.is_primary {
                ButtonStyle::Primary
            } else {
                ButtonStyle::Secondary
            })
            .label(self.button_name.as_str())
            .disabled(self.is_full)
    }
}

#[async_trait]
impl Component for JoinSignupBtn {
    fn custom_id(&self) -> String {
        String::from("signup_join")
    }
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        if let serenity::Interaction::Component(cmp) = i {
            let _ = cmp
                .create_response(
                    ctx.ctx.http,
                    Response::Message(ResponseMsg::new().content(format!(
                        "Cat: {}",
                        ctx.i_ctx.unwrap_or(String::from("None"))
                    ))),
                )
                .await;
        }
    }
}

use std::num::ParseIntError;

use crate::{
    app::logging::log_feature,
    data::signups::queries::join::{UserJoinSignupError, user_join_signup},
    features::signups::{dashboard::format::signup_dashboard, types::full_signup::FullSignup},
    prelude::*,
};
use async_trait::async_trait;
use poise::serenity_prelude::{
    self as serenity, ButtonStyle, ComponentInteraction, CreateButton, Http,
};

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
            let Some(guild_id) = cmp.guild_id else {
                return;
            };
            let user_id = cmp.user.id.get();
            let guild_id = guild_id.get();
            let channel_id = cmp.channel_id.get();
            let message_id = cmp.message.id.get();

            let message_url = format!(
                "https://discord.com/channels/{}/{}/{}",
                guild_id, channel_id, message_id
            );

            let Some(raw_category_id) = ctx.i_ctx else {
                log_signup_error(
                    UserJoinSignupError::Other(
                        format!("Button for signup at {} has missing context", message_url),
                        None,
                    ),
                    &ctx.ctx.http,
                    cmp,
                )
                .await;
                return;
            };

            let Ok(category_id) = raw_category_id
                .parse::<u64>()
                .map_err(|e: ParseIntError| anyhow::anyhow!(e))
            else {
                log_signup_error(
                    UserJoinSignupError::Other(
                        format!("Button for signup at {} has invalid context", message_url),
                        None,
                    ),
                    &ctx.ctx.http,
                    cmp,
                )
                .await;
                return;
            };

            match user_join_signup(&ctx.data.db, category_id, user_id).await {
                Ok(_) => {}
                Err(e) => {
                    log_signup_error(e, &ctx.ctx.http, cmp).await;
                    return;
                }
            }

            let full_signup = match FullSignup::fetch(&ctx.data.db, cmp.message.id.get()).await {
                Ok(s) => s,
                Err(e) => {
                    log_signup_error(
                        UserJoinSignupError::Other(
                            format!("Failed to fetch updated signup in {}", message_url),
                            Some(e.to_string()),
                        ),
                        &ctx.ctx.http,
                        cmp,
                    )
                    .await;
                    return;
                }
            };

            let (embed, components) = signup_dashboard(full_signup).await;

            let _ = cmp
                .create_response(
                    ctx.ctx.http,
                    Response::UpdateMessage(ResponseMsg::new().embed(embed).components(components)),
                )
                .await;
        }
    }
}

async fn log_signup_error(error: UserJoinSignupError, http: &Http, cmp: &ComponentInteraction) {
    let content: String = match error {
        UserJoinSignupError::Other(short, error_trace) => {
            log_feature(LogType::Error, LogFeature::Signup, &short, error_trace);
            short
        }
        UserJoinSignupError::CategoryFull => "Category is full and cannot be joined".into(),
        UserJoinSignupError::NotFound(val) => val.into(),
        UserJoinSignupError::UserAlreadyExists => "You are already in this category".into(),
    };

    let _ = cmp
        .create_response(
            http,
            Response::Message(ResponseMsg::new().content(content).ephemeral(true)),
        )
        .await;
}

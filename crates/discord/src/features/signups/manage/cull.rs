use crate::{
    app::system::components::Component,
    features::signups::{
        embed::fetch_and_format_signup, manage::edit_category::render_edit_category,
    },
    prelude::*,
};

use async_trait::async_trait;
use migrations::tables::SignupMember;
use poise::serenity_prelude::{
    self as serenity, ButtonStyle, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditMessage, MessageId,
};

pub struct SignupCullButton;

impl SignupCullButton {
    pub fn build_with(&self, signup_id: u64, category_id: u64) -> CreateButton {
        CreateButton::new(format!("signup_cull:{signup_id}/{category_id}"))
            .label("Cull Users")
            .style(ButtonStyle::Secondary)
    }
}

#[async_trait]
impl Component for SignupCullButton {
    fn custom_id(&self) -> String {
        String::from("signup_cull")
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        let Some((signup_id, category_id)) = parse_ctx(ctx.i_ctx.as_deref()) else {
            return;
        };
        let Some(guild_id) = component.guild_id else {
            return;
        };

        let roster = match db::Signup::fetch_roster(&ctx.data.db, signup_id).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to fetch roster: {e:?}");
                return;
            }
        };
        let Some(category) = roster.iter().find(|c| c.category_id == category_id) else {
            return;
        };

        let mut removed: u64 = 0;
        for member in &category.members {
            if guild_id
                .member(&ctx.ctx.http, serenity::UserId::new(member.member_id))
                .await
                .is_err()
            {
                match SignupMember::admin_remove(&ctx.data.db, category_id, member.member_id).await
                {
                    Ok(n) => removed += n,
                    Err(e) => {
                        error!("Failed to cull user {}: {e:?}", member.member_id);
                    }
                }
            }
        }

        if removed > 0
            && let Err(e) = refresh_public(&ctx, component.channel_id, signup_id).await
        {
            error!("Failed to refresh public signup after cull: {e:?}");
        }

        let panel = match render_edit_category(&ctx.data.db, signup_id, category_id).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to render edit category after cull: {e:?}");
                return;
            }
        };

        let resp = CreateInteractionResponseMessage::new()
            .content("")
            .embed(panel.embed)
            .components(panel.components);
        let _ = component
            .create_response(
                &ctx.ctx.http,
                CreateInteractionResponse::UpdateMessage(resp),
            )
            .await;
    }
}

pub(super) fn parse_ctx(ctx: Option<&str>) -> Option<(u64, u64)> {
    let raw = ctx?;
    let (a, b) = raw.split_once('/')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

pub(super) async fn refresh_public(
    ctx: &ContextBundle,
    channel_id: serenity::ChannelId,
    signup_id: u64,
) -> Result<(), BotError> {
    let (embed, components) = fetch_and_format_signup(&ctx.data.db, signup_id).await?;
    channel_id
        .edit_message(
            &ctx.ctx.http,
            MessageId::new(signup_id),
            EditMessage::new()
                .content("")
                .embed(embed)
                .components(components),
        )
        .await?;
    Ok(())
}

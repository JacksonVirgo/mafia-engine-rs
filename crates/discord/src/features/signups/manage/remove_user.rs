use crate::{
    app::system::components::Component,
    features::signups::manage::{
        cull::{parse_ctx, refresh_public},
        edit_category::render_edit_category,
    },
    prelude::*,
};

use async_trait::async_trait;
use migrations::tables::SignupMember;
use poise::serenity_prelude::{
    self as serenity, ComponentInteractionDataKind, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
};

pub struct SignupRemoveUserMenu;

impl SignupRemoveUserMenu {
    pub fn build_with(&self, signup_id: u64, category_id: u64) -> CreateSelectMenu {
        CreateSelectMenu::new(
            format!("signup_remove_user:{signup_id}/{category_id}"),
            CreateSelectMenuKind::User {
                default_users: None,
            },
        )
        .placeholder("Select individual users to remove")
        .min_values(1)
        .max_values(25)
    }
}

#[async_trait]
impl Component for SignupRemoveUserMenu {
    fn custom_id(&self) -> String {
        String::from("signup_remove_user")
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        let Some((signup_id, category_id)) = parse_ctx(ctx.i_ctx.as_deref()) else {
            return;
        };
        let ComponentInteractionDataKind::UserSelect { values } = &component.data.kind else {
            return;
        };

        let mut removed_any = false;
        for user_id in values {
            match SignupMember::admin_remove(&ctx.data.db, category_id, user_id.get()).await {
                Ok(n) if n > 0 => removed_any = true,
                Ok(_) => {}
                Err(e) => error!("admin_remove failed: {e:?}"),
            }
        }

        if removed_any && let Err(e) = refresh_public(&ctx, component.channel_id, signup_id).await {
            error!("Failed to refresh public signup after remove: {e:?}");
        }

        let panel = match render_edit_category(&ctx.data.db, signup_id, category_id).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to render edit category after remove: {e:?}");
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

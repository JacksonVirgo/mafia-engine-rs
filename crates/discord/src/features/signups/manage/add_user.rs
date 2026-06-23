use crate::{
    app::system::components::Component,
    features::signups::manage::{
        cull::{parse_ctx, refresh_public},
        edit_category::render_edit_category,
    },
    prelude::*,
};

use async_trait::async_trait;
use migrations::tables::{AdminAddResult, SignupMember};
use poise::serenity_prelude::{
    self as serenity, ComponentInteractionDataKind, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
};

pub struct SignupAddUserMenu;

impl SignupAddUserMenu {
    pub fn build_with(&self, signup_id: u64, category_id: u64) -> CreateSelectMenu {
        CreateSelectMenu::new(
            format!("signup_add_user:{signup_id}/{category_id}"),
            CreateSelectMenuKind::User {
                default_users: None,
            },
        )
        .placeholder("Select individual users to add")
        .min_values(1)
        .max_values(25)
    }
}

#[async_trait]
impl Component for SignupAddUserMenu {
    fn custom_id(&self) -> String {
        String::from("signup_add_user")
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
        let Some(guild_id) = component.guild_id else {
            return;
        };

        let mut notes: Vec<String> = Vec::new();
        let mut added_any = false;
        for user_id in values {
            let username = match guild_id.member(&ctx.ctx.http, *user_id).await {
                Ok(m) => m.user.name.clone(),
                Err(_) => {
                    notes.push(format!("Couldn't fetch <@{}>", user_id.get()));
                    continue;
                }
            };
            match SignupMember::admin_add(&ctx.data.db, category_id, user_id.get(), &username).await
            {
                Ok(AdminAddResult::Added) => added_any = true,
                Ok(AdminAddResult::AlreadyInCategory) => {
                    notes.push(format!("<@{}> already in category", user_id.get()));
                }
                Ok(AdminAddResult::Full) => {
                    notes.push("Category is full".into());
                    break;
                }
                Ok(AdminAddResult::UnknownCategory) => {
                    notes.push("Category not found".into());
                    break;
                }
                Err(e) => {
                    error!("admin_add failed: {e:?}");
                    notes.push(format!("Error adding <@{}>", user_id.get()));
                }
            }
        }

        if added_any && let Err(e) = refresh_public(&ctx, component.channel_id, signup_id).await {
            error!("Failed to refresh public signup after add: {e:?}");
        }

        let panel = match render_edit_category(&ctx.data.db, signup_id, category_id).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to render edit category after add: {e:?}");
                return;
            }
        };

        let mut resp = CreateInteractionResponseMessage::new()
            .content(if notes.is_empty() {
                String::new()
            } else {
                notes.join("\n")
            })
            .embed(panel.embed)
            .components(panel.components);
        if notes.is_empty() {
            resp = resp.content("");
        }
        let _ = component
            .create_response(
                &ctx.ctx.http,
                CreateInteractionResponse::UpdateMessage(resp),
            )
            .await;
    }
}

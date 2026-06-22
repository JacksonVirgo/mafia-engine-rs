use crate::{
    app::system::components::Component,
    features::signups::manage::edit_category::render_edit_category,
    prelude::*,
};

use async_trait::async_trait;
use poise::serenity_prelude::{
    self as serenity, ComponentInteractionDataKind, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

pub struct SignupCategorySelect;

impl SignupCategorySelect {
    pub fn build_with(
        &self,
        signup_id: u64,
        options: Vec<CreateSelectMenuOption>,
    ) -> CreateSelectMenu {
        CreateSelectMenu::new(
            format!("signup_category_select:{signup_id}"),
            CreateSelectMenuKind::String { options },
        )
        .placeholder("Select category to edit")
        .min_values(1)
        .max_values(1)
    }
}

#[async_trait]
impl Component for SignupCategorySelect {
    fn custom_id(&self) -> String {
        String::from("signup_category_select")
    }

    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle) {
        let Some(component) = i.as_message_component() else {
            return;
        };
        let Some(signup_id) = ctx.i_ctx.as_deref().and_then(|s| s.parse::<u64>().ok()) else {
            return;
        };
        let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind else {
            return;
        };
        let Some(category_id) = values.first().and_then(|s| s.parse::<u64>().ok()) else {
            return;
        };

        let panel = match render_edit_category(&ctx.data.db, signup_id, category_id).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to render edit category: {e:?}");
                return;
            }
        };

        let resp = CreateInteractionResponseMessage::new()
            .content("")
            .embed(panel.embed)
            .components(panel.components);
        let _ = component
            .create_response(&ctx.ctx.http, CreateInteractionResponse::UpdateMessage(resp))
            .await;
    }
}

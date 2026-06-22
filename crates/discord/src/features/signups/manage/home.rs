use crate::{app::system::components::Component, prelude::*};

use async_trait::async_trait;
use migrations::Database;
use poise::serenity_prelude::{
    self as serenity, ButtonStyle, Colour, CreateActionRow, CreateButton, CreateEmbed,
    CreateInteractionResponseMessage, CreateSelectMenuOption,
};

use super::category_select::SignupCategorySelect;
use super::playerchats::SignupPlayerChatsButton;

pub struct ManagePanel {
    pub embed: CreateEmbed,
    pub components: Vec<CreateActionRow>,
}

pub async fn render_home(db: &Database, signup_id: u64) -> Result<ManagePanel, BotError> {
    let roster = db::Signup::fetch_roster(db, signup_id).await?;

    let embed = CreateEmbed::new()
        .title("Manage Signup")
        .description("Click the buttons below to edit the signup's settings")
        .colour(Colour::LIGHT_GREY);

    let nav = CreateActionRow::Buttons(vec![
        SignupHomeButton.build_with(signup_id),
        SignupPlayerChatsButton.build_with(signup_id),
    ]);

    let options: Vec<CreateSelectMenuOption> = roster
        .iter()
        .map(|c| {
            CreateSelectMenuOption::new(c.category_name.clone(), c.category_id.to_string())
        })
        .collect();

    let mut components = vec![nav];
    if !options.is_empty() {
        components.push(CreateActionRow::SelectMenu(
            SignupCategorySelect.build_with(signup_id, options),
        ));
    }

    Ok(ManagePanel { embed, components })
}

pub struct SignupHomeButton;

impl SignupHomeButton {
    pub fn build_with(&self, signup_id: u64) -> CreateButton {
        CreateButton::new(format!("signup_home:{signup_id}"))
            .label("Home")
            .style(ButtonStyle::Primary)
    }
}

#[async_trait]
impl Component for SignupHomeButton {
    fn custom_id(&self) -> String {
        String::from("signup_home")
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

pub async fn update_to_home(
    component: &serenity::ComponentInteraction,
    ctx: &ContextBundle,
    signup_id: u64,
) {
    let panel = match render_home(&ctx.data.db, signup_id).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to render signup home: {e:?}");
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
            serenity::CreateInteractionResponse::UpdateMessage(resp),
        )
        .await;
}

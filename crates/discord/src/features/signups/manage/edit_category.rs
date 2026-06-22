use crate::{features::signups::manage::home::ManagePanel, prelude::*};

use migrations::Database;
use migrations::tables::SignupRoster;
use poise::serenity_prelude::{Colour, CreateActionRow, CreateEmbed, CreateEmbedFooter};

use super::add_user::SignupAddUserMenu;
use super::cull::SignupCullButton;
use super::home::SignupHomeButton;
use super::remove_user::SignupRemoveUserMenu;

pub async fn render_edit_category(
    db: &Database,
    signup_id: u64,
    category_id: u64,
) -> Result<ManagePanel, BotError> {
    let roster = db::Signup::fetch_roster(db, signup_id).await?;
    let Some(category) = roster.iter().find(|c| c.category_id == category_id) else {
        return Err(BotError::from(anyhow::anyhow!(
            "category {category_id} not found in signup {signup_id}"
        )));
    };

    let embed = CreateEmbed::new()
        .title(format!("Edit {}", category.category_name))
        .description(format!("Edit the {} category", category.category_name))
        .colour(Colour::LIGHT_GREY)
        .field("Members", member_list(category), false)
        .footer(CreateEmbedFooter::new(format!(
            "category_id: {}",
            category.category_id
        )));

    let nav = CreateActionRow::Buttons(vec![
        SignupHomeButton.build_with(signup_id),
        SignupCullButton.build_with(signup_id, category_id),
    ]);

    let components = vec![
        nav,
        CreateActionRow::SelectMenu(SignupAddUserMenu.build_with(signup_id, category_id)),
        CreateActionRow::SelectMenu(SignupRemoveUserMenu.build_with(signup_id, category_id)),
    ];

    Ok(ManagePanel { embed, components })
}

fn member_list(category: &SignupRoster) -> String {
    if category.members.is_empty() {
        "_empty_".to_string()
    } else {
        category
            .members
            .iter()
            .map(|m| format!("<@{}> ({})", m.member_id, m.username))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

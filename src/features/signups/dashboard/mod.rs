use crate::{
    features::signups::{dashboard::join::JoinSignupBtn, types::full_signup::FullSignup},
    prelude::Button,
};
use poise::serenity_prelude::{Colour, CreateActionRow, CreateButton, CreateEmbed};

pub mod join;
pub mod manage;
pub mod refresh;
pub async fn signup_dashboard(signup: FullSignup) -> (CreateEmbed, Vec<CreateActionRow>) {
    let mut fields: Vec<(String, String, bool)> = vec![];
    let mut hoisted: Vec<(String, String, bool)> = vec![];

    let mut buttons: Vec<CreateButton> = vec![];

    for c in &signup.categories {
        let mut user_ids = Vec::new();
        for slot in signup.slots.iter().filter(|s| s.category_id == c.id) {
            for user in signup.users.iter().filter(|u| u.slot_id == slot.id) {
                user_ids.push(user.user_id);
            }
        }

        let value = if user_ids.is_empty() {
            "> *(nobody)*".to_string()
        } else {
            user_ids
                .iter()
                .map(|id| format!("> <@{}>", id))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let name = match c.max_slots {
            Some(max) => {
                format!("{} [{}/{}]", c.name, user_ids.len(), max)
            }
            _ => {
                if user_ids.len() == 0 {
                    String::from(&c.name)
                } else {
                    format!("{} [{}]", c.name, user_ids.len())
                }
            }
        };

        let field = (name, value, true);

        if c.is_hoisted.0 {
            hoisted.push(field);
        } else {
            fields.push(field);

            let is_full = match c.max_slots {
                None => false,
                Some(limit) => user_ids.len() >= (limit as usize),
            };

            buttons.push(
                JoinSignupBtn {
                    button_name: c.button_name.to_string(),
                    category: c.id,
                    is_primary: buttons.len() == 0,
                    is_full,
                }
                .build()
                .await,
            );
        }
    }

    (
        CreateEmbed::new()
            .title(signup.signup.name)
            .description("Click the appropriate buttons to join a category")
            .color(Colour::BLURPLE)
            .fields(hoisted)
            .field("\u{200B}", "**[__SIGNED UP__]**", false)
            .fields(fields),
        vec![CreateActionRow::Buttons(buttons)],
    )
}

use crate::app::system::components::button::Button;
use crate::features::signups::buttons::{
    leave::SignupLeave, refresh::SignupRefresh, settings::SignupSettings,
};
use crate::prelude::*;
use migrations::Database;
use migrations::tables::SignupRoster;
use serenity::{
    ButtonStyle, Colour, ComponentInteraction, Context as SerenityContext, CreateActionRow,
    CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub async fn fetch_and_format_signup(
    pool: &Database,
    signup_id: u64,
) -> Result<(CreateEmbed, Vec<CreateActionRow>), BotError> {
    let signup = db::Signup::fetch_one(pool, signup_id).await?;
    let roster = db::Signup::fetch_roster(pool, signup_id).await?;

    let mut embed = CreateEmbed::new()
        .title(&signup.name)
        .description("Click the appropriate buttons to join a category")
        .colour(Colour::BLURPLE);

    let (hoisted, unhoisted): (Vec<&SignupRoster>, Vec<&SignupRoster>) =
        roster.iter().partition(|c| c.is_hoisted);

    for category in &hoisted {
        embed = embed.field(category_header(category), category_body(category), true);
    }

    if !hoisted.is_empty() && !unhoisted.is_empty() {
        embed = embed.field("\u{200B}", "**[__SIGNED UP__]**", false);
    }

    let mut join_buttons: Vec<CreateButton> = Vec::new();
    for category in &unhoisted {
        embed = embed.field(category_header(category), category_body(category), true);

        let label = category
            .button_name
            .clone()
            .unwrap_or_else(|| category.category_name.clone());
        let style = if join_buttons.is_empty() {
            ButtonStyle::Primary
        } else {
            ButtonStyle::Secondary
        };
        join_buttons.push(
            CreateButton::new(format!("signup_join:{}", category.category_id))
                .label(label)
                .style(style),
        );
    }

    join_buttons.push(SignupLeave.build().await);
    join_buttons.push(SignupSettings.build().await);

    let rows: Vec<CreateActionRow> = join_buttons
        .chunks(5)
        .map(|chunk| CreateActionRow::Buttons(chunk.to_vec()))
        .collect();

    Ok((embed, rows))
}

fn category_header(category: &SignupRoster) -> String {
    match category.member_limit {
        Some(limit) => format!(
            "{} ({}/{})",
            category.category_name,
            category.members.len(),
            limit
        ),
        None => format!("{} ({})", category.category_name, category.members.len()),
    }
}

fn category_body(category: &SignupRoster) -> String {
    if category.is_anonymous {
        "_anonymous user_".to_string()
    } else if category.members.is_empty() {
        "_empty_".to_string()
    } else {
        category
            .members
            .iter()
            .map(|m| m.username.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub async fn refresh_signup_message(
    component: &ComponentInteraction,
    ctx: &SerenityContext,
    db: &Database,
) -> Result<(), BotError> {
    let (embed, components) = fetch_and_format_signup(db, component.message.id.get()).await?;
    let resp = CreateInteractionResponseMessage::new()
        .content("")
        .embed(embed)
        .components(components);
    component
        .create_response(&ctx.http, CreateInteractionResponse::UpdateMessage(resp))
        .await?;
    Ok(())
}

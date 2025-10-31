use poise::serenity_prelude::{Colour, CreateActionRow, CreateEmbed};

use crate::{features::signups::dashboard::refresh::SignupRefresh, prelude::Button};

pub mod manage;
pub mod refresh;

pub async fn signup_dashboard() -> (CreateEmbed, Vec<CreateActionRow>) {
    (
        CreateEmbed::new()
            .title("Signup")
            .description("Click the appropriate buttons to join a category")
            .color(Colour::BLURPLE)
            .fields(vec![
                ("Hosts", "> TBD", true),
                ("Moderators", "> TBD", true),
                ("Balancers", "> TBD", true),
                ("\u{200B}", "**[__SIGNED UP__]**", false),
                ("Players", "> TBD", true),
                ("Backups", "> TBD", true),
            ]),
        vec![CreateActionRow::Buttons(vec![SignupRefresh.build().await])],
    )
}

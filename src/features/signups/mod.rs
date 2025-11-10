use crate::{
    features::signups::{
        create::create_signups,
        dashboard::{join::JoinSignupBtn, refresh::SignupRefresh, settings::SignupSettings},
    },
    prelude::*,
};

pub mod create;
pub mod dashboard;
pub mod types;

#[poise::command(slash_command, rename = "signups", subcommands("create_signups"))]
pub async fn signups_parent(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

plugin!(SignupPlugin, |app| {
    app.add_commands(vec![signups_parent()]);
    app.add_component(SignupRefresh);
    app.add_component(JoinSignupBtn::default());
    app.add_component(SignupSettings);
});

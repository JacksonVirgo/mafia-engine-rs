use mafia_discord_framework::prelude::*;

pub mod rename;

pub struct CitizenshipPlugin;

impl Plugin for CitizenshipPlugin {
    fn build(&self, app: &mut App) {
        app.add_interaction(rename::rename());
    }
}

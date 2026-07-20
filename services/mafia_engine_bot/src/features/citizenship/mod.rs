use crate::app::database::Database;
use mafia_discord_framework::prelude::*;

pub mod citizen;
pub mod rename;

pub struct CitizenshipPlugin {
    database: Database,
}

impl CitizenshipPlugin {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl Plugin for CitizenshipPlugin {
    fn build(&self, app: &mut App) {
        app.add_interaction(rename::command(self.database.clone()));
        app.add_interaction(citizen::command(self.database.clone()));
    }
}

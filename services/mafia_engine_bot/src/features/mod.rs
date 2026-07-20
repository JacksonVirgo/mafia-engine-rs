use crate::app::database::Database;
use mafia_discord_framework::prelude::*;

pub mod citizenship;

pub struct CorePlugin {
    database: Database,
}

impl CorePlugin {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event_listener(on_ready);
        app.add_plugin(citizenship::CitizenshipPlugin::new(self.database.clone()));
    }
}

async fn on_ready(ready: Ready, _: EventContext) -> Result<(), BoxError> {
    tracing::info!("connected as {}", ready.user.name);
    Ok(())
}

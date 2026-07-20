use mafia_discord_framework::prelude::*;

pub mod citizenship;
pub mod test;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event_listener(on_ready);
        app.add_plugin(citizenship::CitizenshipPlugin);
        test::register(app);
    }
}

async fn on_ready(ready: Ready, _: EventContext) -> Result<(), BoxError> {
    tracing::info!("connected as {}", ready.user.name);
    Ok(())
}

use bot_framework::prelude::*;

pub mod rename;

pub struct CitizenshipPlugin;

impl Plugin<crate::State> for CitizenshipPlugin {
    fn build(&self, app: &mut PluginBuilder<crate::State>) {
        app.add_command::<rename::Rename>();
    }
}

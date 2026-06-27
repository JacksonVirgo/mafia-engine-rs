use bot_framework::prelude::*;

pub mod citizenship;
pub mod config;
pub mod logger;
pub mod ping;

pub struct FeaturePlugin;

impl Plugin<crate::State> for FeaturePlugin {
    fn build(&self, app: &mut PluginBuilder<crate::State>) {
        app.add_plugin(logger::MessageLogPlugin);
        app.add_plugin(citizenship::CitizenshipPlugin);
    }
}

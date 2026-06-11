pub mod heartbeat;
use crate::prelude::*;

plugin!(FeaturePlugin, |app| {
    app.add_command(heartbeat::heartbeat());
});

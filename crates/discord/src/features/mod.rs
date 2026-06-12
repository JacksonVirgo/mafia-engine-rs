pub mod heartbeat;
pub mod middleware;
pub mod rename;
pub mod test_admin;

use crate::{
    features::middleware::{require_member::RequireMember, require_server::RequireServer},
    prelude::*,
};

plugin!(FeaturePlugin, |app| {
    app.add_command(heartbeat::heartbeat().with(RequireServer));
    app.add_command(rename::rename().with(RequireMember));
    app.add_plugin(test_admin::TestAdminPlugin);
});

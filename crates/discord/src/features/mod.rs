pub mod archiving;
pub mod heartbeat;
pub mod middleware;
pub mod playerchats;
pub mod rename;
pub mod signups;
pub mod test_admin;
pub mod votecounter;

use crate::{
    features::middleware::{require_member::RequireMember, require_server::RequireServer},
    prelude::*,
};

plugin!(FeaturePlugin, |app| {
    app.add_command(heartbeat::heartbeat().with(RequireServer));
    app.add_command(rename::rename().with(RequireMember));
    app.add_plugin(test_admin::TestAdminPlugin);

    app.add_plugin(signups::SignupPlugin);
    app.add_plugin(votecounter::VoteCounterPlugin);
});

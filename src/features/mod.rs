use crate::prelude::*;

pub mod citizenship;
pub mod signups;
pub mod testing;

plugin!(FeaturePlugin, |app| {
    #[cfg(debug_assertions)]
    app.add_plugin(testing::TestingPlugin);

    app.add_plugin(citizenship::CitizenshipPlugin)
        .add_plugin(signups::SignupPlugin);
});

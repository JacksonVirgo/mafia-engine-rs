use crate::{
    features::testing::{command::test, component::TestingButton},
    prelude::*,
};

pub mod testing;

plugin!(FeaturePlugin, |app| {
    info!("Used plugin");
    app.add_command(test()).add_component("test", TestingButton);
});

use crate::prelude::*;

pub mod command;
pub mod component;

plugin!(TestingPlugin, |app| {
    app.add_command(command::test())
        .add_component("test", component::TestingButton);
});

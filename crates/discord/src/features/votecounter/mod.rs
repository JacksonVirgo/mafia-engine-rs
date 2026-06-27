use crate::prelude::*;

pub mod manage;

plugin!(VoteCounterPlugin, |app| {
    app.add_command(manage::manage());
});

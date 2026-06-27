use crate::prelude::*;

pub mod manage;
pub mod players;

plugin!(VoteCounterPlugin, |app| {
    app.add_commands(vec![
        manage::manage(),
        players::vote::vote(),
        players::unvote::unvote(),
        players::skip::skip(),
        players::votecount::votecount(),
    ]);
});

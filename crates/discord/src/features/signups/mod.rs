pub mod create;

use crate::{features::middleware::require_server::RequireServer, prelude::*};
use create::create;

#[poise::command(slash_command, subcommands("create"), subcommand_required)]
pub async fn signups(_: BotCtx<'_>) -> Result<(), BotError> {
    Ok(())
}

plugin!(SignupPlugin, |app| {
    let mut cmd = signups();
    cmd.subcommands = vec![create::create().with(RequireServer)];
    app.add_command(cmd);
});

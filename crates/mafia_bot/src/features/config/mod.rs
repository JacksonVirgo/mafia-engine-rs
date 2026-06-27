use bot_framework::prelude::*;

mod get_prefix;
mod set_prefix;

#[command_group(
    name = "config",
    description = "Server configuration",
    subcommands(set_prefix::SetPrefix, get_prefix::GetPrefix),
)]
pub struct Config;

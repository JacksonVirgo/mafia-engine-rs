extern crate self as mafia_discord_framework;

mod app;
mod plugin;
mod slash_command;

pub use mafia_discord_framework_macros::slash_command;
pub use {
    app::{App, BoxError, EventContext, EventPayload},
    plugin::Plugin,
    slash_command::{CommandContext, CommandOptionError, SlashCommand},
};

pub mod prelude {
    pub use crate::{
        App, BoxError, CommandContext, CommandOptionError, EventContext, EventPayload, Plugin,
        SlashCommand,
    };
    pub use mafia_discord_framework_macros::slash_command;
    pub use twilight_gateway::{EventTypeFlags, Intents};
    pub use twilight_model::gateway::{
        event::Event,
        payload::incoming::{InteractionCreate, MessageCreate, Ready},
    };
}

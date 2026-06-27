pub mod app;
pub mod commands;
pub mod error;
pub mod plugin;

pub use bot_framework_macros::{command, command_group, subcommand, subcommand_group};

pub mod prelude {
    pub use crate::{command, command_group, subcommand, subcommand_group};
    pub use crate::{
        app::{Bot, BotData, EventListener},
        commands::{
            CommandCtx, CommandDescriptor, CommandKind, CommandModule, FromInteractionOptions,
            LeafHandler, SubcommandDescriptor, SubcommandEntry, SubcommandGroupDescriptor,
            SubcommandGroupModule, SubcommandModule,
        },
        error::{BotError, Rejection},
        plugin::{Plugin, PluginBuilder},
    };
    pub use async_trait::async_trait;
    pub use twilight_gateway::{Event, EventTypeFlags, Intents};
}

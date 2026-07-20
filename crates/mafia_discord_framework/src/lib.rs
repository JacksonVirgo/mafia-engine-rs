extern crate self as mafia_discord_framework;

mod app;
mod component;
mod context;
mod plugin;
mod slash_command;

pub use mafia_discord_framework_macros::slash_command;
pub use {
    app::{App, BoxError, EventContext, EventPayload},
    component::{
        ActionRowBuilder, Button, ButtonData, ChannelSelect, ChannelSelectData, Component,
        ComponentBuilder, ComponentContext, DiscordComponent, MentionableSelect,
        MentionableSelectData, RegisteredComponent, RoleSelect, RoleSelectData, TextSelect,
        TextSelectData, UserSelect, UserSelectData, select_option,
    },
    context::GlobalContext,
    plugin::Plugin,
    slash_command::{CommandContext, CommandOptionError, SlashCommand},
};

pub mod prelude {
    pub use crate::{
        ActionRowBuilder, App, BoxError, Button, ButtonData, ChannelSelect, ChannelSelectData,
        CommandContext, CommandOptionError, Component, ComponentBuilder, ComponentContext,
        DiscordComponent, EventContext, EventPayload, GlobalContext, MentionableSelect,
        MentionableSelectData, Plugin, RegisteredComponent, RoleSelect, RoleSelectData,
        SlashCommand, TextSelect, TextSelectData, UserSelect, UserSelectData, select_option,
    };
    pub use mafia_discord_framework_macros::slash_command;
    pub use twilight_gateway::{EventTypeFlags, Intents};
    pub use twilight_model::channel::message::component::ButtonStyle;
    pub use twilight_model::gateway::{
        event::Event,
        payload::incoming::{InteractionCreate, MessageCreate, Ready},
    };
}

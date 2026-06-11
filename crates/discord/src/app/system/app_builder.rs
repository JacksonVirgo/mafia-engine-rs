use crate::{
    app::{
        discord::{BotState, BotError},
        system::registry::{COMPONENT_REGISTRY, IntoComponentPtr},
    },
    prelude::Plugin,
};

pub struct AppBuilder<'a> {
    pub commands: &'a mut Vec<poise::Command<BotState, BotError>>,
}

impl<'a> AppBuilder<'a> {
    pub fn add_plugin<P>(&mut self, plugin: P) -> &mut Self
    where
        P: Plugin,
    {
        plugin.build(self);
        self
    }

    pub fn add_plugins<I, P>(&mut self, plugins: I) -> &mut Self
    where
        I: IntoIterator<Item = P>,
        P: Plugin,
    {
        for plugin in plugins {
            plugin.build(self);
        }
        self
    }

    pub fn add_component<T>(&mut self, comp: T) -> &mut Self
    where
        T: IntoComponentPtr,
    {
        let ptr = comp.shared();
        COMPONENT_REGISTRY.insert(ptr.custom_id(), ptr);
        self
    }

    pub fn add_command(&mut self, cmd: poise::Command<BotState, BotError>) -> &mut Self {
        self.commands.push(cmd);
        self
    }

    pub fn add_commands<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = poise::Command<BotState, BotError>>,
    {
        self.commands.extend(iter);
        self
    }
    pub fn add_command_with<F>(&mut self, make: F) -> &mut Self
    where
        F: FnOnce() -> poise::Command<BotState, BotError>,
    {
        self.commands.push(make());
        self
    }
}

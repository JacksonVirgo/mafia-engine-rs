use crate::{
    app::{
        discord::{BotError, BotState},
        system::{
            middleware::{DynMiddleware, expand_stack},
            registry::{COMPONENT_REGISTRY, IntoComponentEntry},
        },
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
        T: IntoComponentEntry,
    {
        let entry = comp.into_entry();
        let key = entry.component.custom_id();
        COMPONENT_REGISTRY.insert(key, entry);
        self
    }

    pub fn add_command(&mut self, mut cmd: poise::Command<BotState, BotError>) -> &mut Self {
        if let Some(list) = cmd.custom_data.downcast_mut::<Vec<DynMiddleware>>() {
            let owned = std::mem::take(list);
            *list = expand_stack(owned);
        }
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

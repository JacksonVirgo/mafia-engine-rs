use crate::app::{
    discord::{ContextData, Error},
    system::registry::{COMPONENT_REGISTRY, IntoComponentPtr},
};

pub struct AppBuilder<'a> {
    pub commands: &'a mut Vec<poise::Command<ContextData, Error>>,
}

impl<'a> AppBuilder<'a> {
    pub fn add_component<T>(&mut self, key: impl Into<String>, comp: T) -> &mut Self
    where
        T: IntoComponentPtr,
    {
        let ptr = comp.shared();
        COMPONENT_REGISTRY.insert(key.into(), ptr);
        self
    }

    pub fn add_command(&mut self, cmd: poise::Command<ContextData, Error>) -> &mut Self {
        self.commands.push(cmd);
        self
    }

    pub fn add_commands<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = poise::Command<ContextData, Error>>,
    {
        self.commands.extend(iter);
        self
    }
    pub fn add_command_with<F>(&mut self, make: F) -> &mut Self
    where
        F: FnOnce() -> poise::Command<ContextData, Error>,
    {
        self.commands.push(make());
        self
    }
}

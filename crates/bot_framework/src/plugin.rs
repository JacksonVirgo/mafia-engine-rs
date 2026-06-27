use std::sync::Arc;

use twilight_gateway::EventTypeFlags;

use crate::app::EventListener;
use crate::commands::{CommandDescriptor, CommandModule};

pub trait Plugin<S: Send + Sync + 'static>: Send + Sync + 'static {
    fn build(&self, app: &mut PluginBuilder<S>);
}

pub struct PluginBuilder<S: Send + Sync + 'static> {
    pub(crate) listeners: Vec<RegisteredListener<S>>,
    pub(crate) commands: Vec<CommandDescriptor<S>>,
}

pub(crate) struct RegisteredListener<S: Send + Sync + 'static> {
    pub filter: EventTypeFlags,
    pub handler: Arc<dyn EventListener<S>>,
}

impl<S: Send + Sync + 'static> PluginBuilder<S> {
    pub fn add_plugin<P: Plugin<S>>(&mut self, p: P) -> &mut Self {
        p.build(self);
        self
    }

    pub fn add_plugins<I, P>(&mut self, plugins: I) -> &mut Self
    where
        I: IntoIterator<Item = P>,
        P: Plugin<S>,
    {
        for p in plugins {
            p.build(self);
        }
        self
    }

    pub fn add_listener<L: EventListener<S>>(
        &mut self,
        events: EventTypeFlags,
        listener: L,
    ) -> &mut Self {
        self.listeners.push(RegisteredListener {
            filter: events,
            handler: Arc::new(listener),
        });
        self
    }

    pub fn add_command<C: CommandModule<S>>(&mut self) -> &mut Self {
        self.commands.push(C::descriptor());
        self
    }
}

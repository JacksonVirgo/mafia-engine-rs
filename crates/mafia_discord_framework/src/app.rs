use crate::plugin::Plugin;
use std::{any::TypeId, collections::HashSet};

pub struct App {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_types: HashSet<TypeId>,
    state: PluginState,
    systems: Vec<Box<dyn FnMut()>>,
}

#[derive(Default, PartialEq)]
enum PluginState {
    #[default]
    Adding,
    Finished,
    Cleaned,
}

impl Default for App {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_types: HashSet::new(),
            state: PluginState::Adding,
            systems: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        assert!(
            self.state == PluginState::Adding,
            "plugins can no longer be added"
        );

        let id = TypeId::of::<P>();

        assert!(
            self.plugin_types.insert(id),
            "plugin {} was already added",
            std::any::type_name::<P>()
        );

        plugin.build(self);

        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn add_system(&mut self, system: impl FnMut() + 'static) -> &mut Self {
        self.systems.push(Box::new(system));
        self
    }

    pub fn run(mut self) {
        while !self.plugins.iter().all(|plugin| plugin.ready(&self)) {
            std::thread::yield_now();
        }

        self.call_finish();
        self.call_cleanup();

        loop {
            for system in &mut self.systems {
                system();
            }
        }
    }

    fn call_finish(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);
        for plugin in &plugins {
            plugin.finish(self);
        }
        self.plugins = plugins;
        self.state = PluginState::Finished;
    }

    fn call_cleanup(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);
        for plugin in &plugins {
            plugin.cleanup(self);
        }
        self.plugins = plugins;
        self.state = PluginState::Cleaned;
    }
}

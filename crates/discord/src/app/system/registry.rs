use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;

use crate::prelude::*;

pub type ComponentPtr = Arc<dyn Component + Send + Sync>;

pub trait IntoComponentPtr: Component + Sized + Send + Sync + 'static {
    fn shared(self) -> ComponentPtr {
        Arc::new(self)
    }
}

impl<T: Component + Sized + Send + Sync + 'static> IntoComponentPtr for T {}

pub static COMPONENT_REGISTRY: Lazy<DashMap<String, ComponentPtr>> = Lazy::new(DashMap::new);

pub fn register_component(key: impl Into<String>, comp: impl Into<ComponentPtr>) {
    COMPONENT_REGISTRY.insert(key.into(), comp.into());
}

pub fn get_component(key: &str) -> Option<ComponentPtr> {
    COMPONENT_REGISTRY.get(key).map(|entry| entry.clone())
}

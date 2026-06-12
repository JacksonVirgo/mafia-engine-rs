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

#[derive(Clone)]
pub struct ComponentEntry {
    pub component: ComponentPtr,
    pub middleware: Arc<[DynMiddleware]>,
}

pub static COMPONENT_REGISTRY: Lazy<DashMap<String, ComponentEntry>> = Lazy::new(DashMap::new);

pub fn register_component(
    key: impl Into<String>,
    comp: impl Into<ComponentPtr>,
    middleware: impl IntoIterator<Item = DynMiddleware>,
) {
    COMPONENT_REGISTRY.insert(
        key.into(),
        ComponentEntry {
            component: comp.into(),
            middleware: middleware.into_iter().collect::<Vec<_>>().into(),
        },
    );
}

pub fn get_component(key: &str) -> Option<ComponentEntry> {
    COMPONENT_REGISTRY.get(key).map(|entry| entry.clone())
}

pub trait IntoComponentEntry {
    fn into_entry(self) -> ComponentEntry;
}

impl<T> IntoComponentEntry for T
where
    T: IntoComponentPtr,
{
    fn into_entry(self) -> ComponentEntry {
        ComponentEntry {
            component: self.shared(),
            middleware: Arc::from(Vec::<DynMiddleware>::new()),
        }
    }
}

impl<T> IntoComponentEntry for super::middleware::WithMiddleware<T>
where
    T: IntoComponentPtr,
{
    fn into_entry(self) -> ComponentEntry {
        let expanded = super::middleware::expand_stack(self.middleware);
        ComponentEntry {
            component: self.inner.shared(),
            middleware: Arc::from(expanded),
        }
    }
}

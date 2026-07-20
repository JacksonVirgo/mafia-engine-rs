use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Shared application state available to plugins and all event handlers.
///
/// Values are addressed by their Rust type. Store application-wide services during setup, then
/// retrieve them from a handler with [`GlobalContext::get`].
#[derive(Clone, Default)]
pub struct GlobalContext {
    values: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl GlobalContext {
    /// Stores a value for its type, replacing any value of that type already in the context.
    pub fn insert<T>(&self, value: T)
    where
        T: Send + Sync + 'static,
    {
        self.values
            .write()
            .expect("global context lock was poisoned")
            .insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Returns the value stored for `T`, if one has been registered.
    pub fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.values
            .read()
            .expect("global context lock was poisoned")
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(|value| value.downcast::<T>().ok())
    }

    /// Removes and returns the value stored for `T`, if one exists.
    pub fn remove<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.values
            .write()
            .expect("global context lock was poisoned")
            .remove(&TypeId::of::<T>())
            .and_then(|value| value.downcast::<T>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::GlobalContext;

    #[test]
    fn values_are_shared_by_type() {
        let context = GlobalContext::default();
        context.insert(String::from("database"));

        assert_eq!(
            context
                .get::<String>()
                .map(|value| value.as_str().to_owned()),
            Some("database".to_owned())
        );
        assert!(context.get::<u64>().is_none());
    }
}

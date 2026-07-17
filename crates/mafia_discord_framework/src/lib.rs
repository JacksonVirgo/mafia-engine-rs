mod app;
mod plugin;

pub mod prelude {
    pub use crate::{
        app::{App, BoxError, EventContext, EventPayload},
        plugin::Plugin,
    };
    pub use twilight_gateway::{EventTypeFlags, Intents};
    pub use twilight_model::gateway::{
        event::Event,
        payload::incoming::{InteractionCreate, MessageCreate, Ready},
    };
}

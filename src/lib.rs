pub mod app;
pub mod data;
pub mod features;

pub mod prelude {
    pub use super::{
        app::{
            discord::{Context, ContextData, Error},
            system::{
                app_builder::AppBuilder,
                component::{Component, ContextBundle, button::Button},
                plugin::Plugin,
            },
        },
        plugin,
    };
    pub use poise::serenity_prelude as serenity;
    pub use tracing::{debug, error, info, warn};
}

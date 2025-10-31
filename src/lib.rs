pub mod app;
pub mod data;
pub mod features;

pub mod prelude {
    pub use super::{
        app::{
            database::Database,
            discord::{Context, ContextData, Error},
            logging::{LogFeature, LogType},
            system::{
                app_builder::AppBuilder,
                component::{Component, ContextBundle, button::Button},
                plugin::Plugin,
            },
        },
        data::prelude::*,
        plugin,
    };
    pub use poise::serenity_prelude as serenity;
    pub use tracing::{debug, error, info, warn};
}

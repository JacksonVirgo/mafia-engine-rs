pub mod app;
pub mod features;

pub mod prelude {
    pub use poise::serenity_prelude::{self as serenity};
    pub use tracing::{debug, error, info, warn};

    pub use super::{
        app::{
            discord::{BotCtx, BotError, BotState},
            system::{
                app_builder::AppBuilder,
                components::{Component, ContextBundle},
                plugin::Plugin,
            },
        },
        plugin,
    };
}

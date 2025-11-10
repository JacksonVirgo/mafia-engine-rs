pub mod app;
pub mod data;
pub mod features;

pub mod prelude {
    pub use super::{
        app::{
            database::Database,
            discord::{Context, ContextData, Error},
            logging::{LogType, features::LogFeature},
            system::{
                app_builder::AppBuilder,
                component::{Component, ContextBundle, button::Button},
                plugin::Plugin,
            },
        },
        data::prelude::*,
        plugin,
    };
    pub use poise::serenity_prelude::{
        self as serenity, CreateInteractionResponse as Response,
        CreateInteractionResponseMessage as ResponseMsg,
    };
    pub use tracing::{debug, error, info, warn};
}

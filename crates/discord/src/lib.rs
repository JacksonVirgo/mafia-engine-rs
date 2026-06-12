pub mod app;
pub mod features;

pub mod prelude {
    pub use migrations::tables as db;
    pub use poise::serenity_prelude::{self as serenity};
    pub use tracing::{debug, error, info, warn};

    pub use super::{
        app::{
            discord::{BotCtx, BotError, BotState},
            system::{
                app_builder::AppBuilder,
                components::{Component, ContextBundle},
                middleware::{
                    CommandMiddlewareExt, ComponentMiddlewareExt, DynMiddleware, Extensions,
                    Middleware, Next, Outcome, Rejection, Request, RequestKind, WithMiddleware,
                    ext,
                },
                plugin::Plugin,
            },
        },
        plugin,
    };
}

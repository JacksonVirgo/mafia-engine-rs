use crate::app::discord::{ContextData, Error};
use poise::serenity_prelude::{self as serenity, FullEvent};

pub async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, ContextData, Error>,
    _data: &ContextData,
) -> Result<(), Error> {
    match event {
        FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }
        }
        _ => {}
    }

    Ok(())
}

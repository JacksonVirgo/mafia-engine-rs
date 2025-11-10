use crate::{app::system::registry::get_component, prelude::*};

use poise::serenity_prelude::{self as serenity, FullEvent, InteractionType};
use tracing::info;

pub async fn event_handler<'a>(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, ContextData, Error>,
    data: &ContextData,
) -> Result<(), Error> {
    match event {
        FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }
        }
        FullEvent::InteractionCreate { interaction } => match interaction.kind() {
            InteractionType::Component => {
                let Some(i) = interaction.as_message_component() else {
                    return Ok(());
                };

                let (cid, ctx_str) = i
                    .data
                    .custom_id
                    .split_once(':')
                    .map(|(id, ctx)| (id, Some(ctx.to_string())))
                    .unwrap_or((i.data.custom_id.as_str(), None));

                let cmp = get_component(&cid);

                match cmp {
                    None => {
                        info!("Could not find component: {}", i.data.custom_id);
                    }
                    Some(cmp) => {
                        let _ = cmp
                            .run(
                                interaction,
                                ContextBundle {
                                    ctx: ctx.clone(),
                                    data: data.clone(),
                                    i_ctx: ctx_str,
                                },
                            )
                            .await;
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

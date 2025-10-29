use crate::app::{
    component::ContextBundle,
    discord::{ContextData, Error},
    system::registry::get_component,
};
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

                let cmp = get_component(&i.data.custom_id);

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
                                },
                            )
                            .await;
                    }
                }

                // let cmp = data.registry.get(i.data.custom_id.as_str());

                // match cmp {
                //     Some(component) => {
                //         let bundle = ContextBundle {
                //             ctx: ctx.clone(),
                //             data: data.clone(),
                //         };
                //     }
                //     None => {
                //         info!("Did not find component");
                //     }
                // }
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

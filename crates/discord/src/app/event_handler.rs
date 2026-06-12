use crate::{
    app::{
        discord::{BotError, BotState},
        system::{middleware::run_stack, registry::get_component},
    },
    prelude::*,
};

use poise::serenity_prelude::{self as serenity, FullEvent, InteractionType};

pub async fn event_handler<'a>(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, BotState, BotError>,
    data: &BotState,
) -> Result<(), BotError> {
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
                    .map(|(id, c)| (id.to_string(), Some(c.to_string())))
                    .unwrap_or((i.data.custom_id.clone(), None));

                let Some(entry) = get_component(&cid) else {
                    info!("Could not find component: {}", i.data.custom_id);
                    return Ok(());
                };

                let mut req = Request {
                    kind: RequestKind::Component {
                        custom_id: cid.clone(),
                        i_ctx: ctx_str.clone(),
                    },
                    serenity_ctx: ctx.clone(),
                    data: data.clone(),
                    user_id: i.user.id,
                    guild_id: i.guild_id,
                    channel_id: i.channel_id,
                    ext: Extensions::default(),
                };

                if !entry.middleware.is_empty() {
                    match run_stack(&entry.middleware, &mut req).await? {
                        Outcome::Continue => {}
                        Outcome::Reject(rej) => {
                            let resp = serenity::CreateInteractionResponseMessage::new()
                                .content(rej.message)
                                .ephemeral(rej.ephemeral);
                            let _ = i
                                .create_response(
                                    &ctx.http,
                                    serenity::CreateInteractionResponse::Message(resp),
                                )
                                .await;
                            return Ok(());
                        }
                    }
                }

                let _ = entry
                    .component
                    .run(
                        interaction,
                        ContextBundle {
                            ctx: ctx.clone(),
                            data: data.clone(),
                            i_ctx: ctx_str,
                            ext: req.ext,
                        },
                    )
                    .await;
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

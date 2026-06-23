use crate::{
    app::{
        discord::{BotError, BotState},
        system::{middleware::run_stack, registry::get_component},
    },
    prelude::*,
};

use poise::serenity_prelude::{self as serenity, FullEvent, InteractionType};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, BotState, BotError>,
    data: &BotState,
) -> Result<(), BotError> {
    match event {
        FullEvent::Message { new_message } if new_message.author.bot => {
            return Ok(());
        }

        FullEvent::InteractionCreate { interaction } => {
            let (custom_id, user_id, guild_id, channel_id) = match interaction.kind() {
                InteractionType::Component => {
                    let Some(i) = interaction.as_message_component() else {
                        return Ok(());
                    };
                    (
                        i.data.custom_id.clone(),
                        i.user.id,
                        i.guild_id,
                        i.channel_id,
                    )
                }
                InteractionType::Modal => {
                    let Some(m) = interaction.as_modal_submit() else {
                        return Ok(());
                    };
                    (
                        m.data.custom_id.clone(),
                        m.user.id,
                        m.guild_id,
                        m.channel_id,
                    )
                }
                _ => return Ok(()),
            };

            let (cid, ctx_str) = match custom_id.split_once(':') {
                Some((id, c)) => (id.to_string(), Some(c.to_string())),
                None => (custom_id.clone(), None),
            };

            let Some(entry) = get_component(&cid) else {
                info!("Could not find component: {}", custom_id);
                return Ok(());
            };

            let mut req = Request {
                kind: RequestKind::Component {
                    custom_id: cid.clone(),
                    i_ctx: ctx_str.clone(),
                },
                serenity_ctx: ctx.clone(),
                data: data.clone(),
                user_id,
                guild_id,
                channel_id,
                ext: Extensions::default(),
            };

            if !entry.middleware.is_empty() {
                match run_stack(&entry.middleware, &mut req).await? {
                    Outcome::Continue => {}
                    Outcome::Reject(rej) => {
                        let resp = serenity::CreateInteractionResponseMessage::new()
                            .content(rej.message)
                            .ephemeral(rej.ephemeral);
                        let response = serenity::CreateInteractionResponse::Message(resp);
                        match interaction.kind() {
                            InteractionType::Component => {
                                if let Some(i) = interaction.as_message_component() {
                                    let _ = i.create_response(&ctx.http, response).await;
                                }
                            }
                            InteractionType::Modal => {
                                if let Some(m) = interaction.as_modal_submit() {
                                    let _ = m.create_response(&ctx.http, response).await;
                                }
                            }
                            _ => {}
                        }
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
    }

    Ok(())
}

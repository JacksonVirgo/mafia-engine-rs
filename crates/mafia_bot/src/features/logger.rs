use bot_framework::prelude::*;

pub struct MessageLogPlugin;

impl Plugin<()> for MessageLogPlugin {
    fn build(&self, app: &mut PluginBuilder<()>) {
        app.add_listener(
            EventTypeFlags::MESSAGE_CREATE
                | EventTypeFlags::MESSAGE_UPDATE
                | EventTypeFlags::MESSAGE_DELETE
                | EventTypeFlags::MESSAGE_DELETE_BULK,
            MessageLoggerHandler,
        );
    }
}

struct MessageLoggerHandler;

#[async_trait]
impl EventListener<()> for MessageLoggerHandler {
    async fn on_event(&self, event: Event, _bot: BotData<()>) -> Result<(), BotError> {
        match event {
            Event::MessageCreate(msg) => {
                tracing::info!(
                    channel_id = %msg.channel_id,
                    author = %msg.author.name,
                    message_id = %msg.id,
                    content = %msg.content,
                    "message create",
                );
            }
            Event::MessageUpdate(msg) => {
                tracing::info!(
                    channel_id = %msg.channel_id,
                    message_id = %msg.id,
                    content = ?msg.content,
                    "message update",
                );
            }
            Event::MessageDelete(msg) => {
                tracing::info!(
                    channel_id = %msg.channel_id,
                    message_id = %msg.id,
                    guild_id = ?msg.guild_id,
                    "message delete",
                );
            }
            Event::MessageDeleteBulk(msg) => {
                tracing::info!(
                    channel_id = %msg.channel_id,
                    count = msg.ids.len(),
                    guild_id = ?msg.guild_id,
                    "message delete bulk",
                );
            }
            _ => {}
        }
        Ok(())
    }
}

use bot_framework::prelude::*;

#[command(name = "ping", description = "Reply with pong")]
pub struct Ping;

impl Ping {
    async fn run(self, ctx: CommandCtx<crate::State>) -> Result<(), BotError> {
        ctx.respond_ephemeral("pong").await
    }
}

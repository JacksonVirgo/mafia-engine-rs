use bot_framework::prelude::*;

#[subcommand(name = "get-prefix", description = "Show the current command prefix")]
pub struct GetPrefix;

impl GetPrefix {
    async fn run(self, ctx: CommandCtx<crate::State>) -> Result<(), BotError> {
        ctx.respond_ephemeral("prefix is `!`").await
    }
}

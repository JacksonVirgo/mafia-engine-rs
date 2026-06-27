use bot_framework::prelude::*;

#[subcommand(name = "set-prefix", description = "Set the bot's command prefix")]
pub struct SetPrefix {
    /// The new prefix to use
    pub prefix: String,
}

impl SetPrefix {
    async fn run(self, ctx: CommandCtx<crate::State>) -> Result<(), BotError> {
        ctx.respond_ephemeral(format!("prefix set to `{}`", self.prefix))
            .await
    }
}

use bot_framework::prelude::*;

#[command(name = "rename", description = "Change your username")]
pub struct Rename {
    username: String,
}

impl Rename {
    async fn run(self, ctx: CommandCtx<crate::State>) -> Result<(), BotError> {
        ctx.respond_ephemeral(format!("TBD: {}", self.username))
            .await
    }
}

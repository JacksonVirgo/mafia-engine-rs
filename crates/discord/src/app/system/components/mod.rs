use crate::app::discord::BotState;
use crate::prelude::*;
use async_trait::async_trait;

pub struct ContextBundle {
    pub ctx: serenity::Context,
    pub data: BotState,
    pub i_ctx: Option<String>,
}

#[async_trait]
pub trait Component {
    fn custom_id(&self) -> String;
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle);
}

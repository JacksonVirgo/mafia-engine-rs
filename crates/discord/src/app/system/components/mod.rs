use crate::app::discord::BotState;
use crate::prelude::*;
use async_trait::async_trait;

pub struct ContextBundle {
    pub ctx: serenity::Context,
    pub data: BotState,
    pub i_ctx: Option<String>,
    pub ext: Extensions,
}

impl ContextBundle {
    pub fn ext<T: Send + Sync + 'static>(&self) -> Result<&T, BotError> {
        self.ext.get::<T>()
    }
}

#[async_trait]
pub trait Component {
    fn custom_id(&self) -> String;
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle);
}

use crate::app::discord::ContextData;
use crate::prelude::*;
use async_trait::async_trait;

pub mod button;

pub struct ContextBundle {
    pub ctx: serenity::Context,
    pub data: ContextData,
    pub i_ctx: Option<String>,
}

#[async_trait]
pub trait Component {
    fn custom_id(&self) -> String;
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle);
}

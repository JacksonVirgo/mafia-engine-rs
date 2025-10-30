use crate::app::discord::ContextData;
use crate::prelude::*;
use async_trait::async_trait;

pub mod button;

pub struct ContextBundle {
    pub ctx: serenity::Context,
    pub data: ContextData,
}

#[async_trait]
pub trait Component {
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle);
}

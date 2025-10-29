use async_trait::async_trait;
use poise::serenity_prelude as serenity;

use crate::app::discord::ContextData;

pub struct ContextBundle {
    pub ctx: serenity::Context,
    pub data: ContextData,
}

#[async_trait]
pub trait Component {
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle);
}

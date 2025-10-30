use crate::app::discord::ContextData;
use crate::prelude::*;
use async_trait::async_trait;
use poise::serenity_prelude::json::{json, to_value};
use poise::serenity_prelude::{ButtonStyle, ComponentType};
use serde_json::Value;

pub struct ContextBundle {
    pub ctx: serenity::Context,
    pub data: ContextData,
}

#[async_trait]
pub trait Component {
    async fn run(&self, i: &serenity::Interaction, ctx: ContextBundle);
}

#[async_trait]
pub trait Button {
    async fn build(&self) -> serenity::CreateButton;

    async fn build_as_json(&self) -> anyhow::Result<Value> {
        let build = self.build().await;
        let val = to_value(&build)?;

        let style = val.get("style").and_then(Value::as_u64).unwrap_or(1) as u8;
        let style = ButtonStyle::from(style);

        let label = val.get("label").and_then(Value::as_str);
        let custom_id = val.get("custom_id").and_then(Value::as_str);

        let disabled = val
            .get("disabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let body = json!({
           "type": ComponentType::Button,
           "style": style,
           "label": label,
           "custom_id": custom_id,
           "disabled": disabled
        });

        Ok(body)
    }
}

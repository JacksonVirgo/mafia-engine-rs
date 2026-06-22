use crate::prelude::*;
use async_trait::async_trait;

#[async_trait]
pub trait SelectMenu {
    async fn build(&self) -> serenity::CreateSelectMenu;
}

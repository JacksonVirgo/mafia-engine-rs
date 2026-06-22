use crate::prelude::*;
use async_trait::async_trait;

#[async_trait]
pub trait Modal {
    async fn build(&self) -> serenity::CreateModal;
}

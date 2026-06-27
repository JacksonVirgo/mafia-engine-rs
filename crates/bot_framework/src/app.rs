use async_trait::async_trait;
use std::sync::Arc;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId};
pub use twilight_gateway::{Event as GatewayEvent, Intents as GatewayIntents, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_model::id::{Id, marker::ApplicationMarker};

use crate::{
    error::BotError,
    plugin::{Plugin, PluginBuilder},
};

pub struct Bot<S: Send + Sync + 'static> {
    token: String,
    intents: Intents,
    state: Arc<S>,
    plugins: Vec<Box<dyn Plugin<S>>>,
}

pub struct BotData<S: Send + Sync + 'static> {
    pub http: Arc<HttpClient>,
    pub state: Arc<S>,
    pub application_id: Id<ApplicationMarker>,
}

impl<S: Send + Sync + 'static> BotData<S> {
    pub fn interaction(&self) -> twilight_http::client::InteractionClient<'_> {
        self.http.interaction(self.application_id)
    }
}

impl<S: Send + Sync + 'static> Clone for BotData<S> {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            state: self.state.clone(),
            application_id: self.application_id,
        }
    }
}

#[async_trait]
pub trait EventListener<S: Send + Sync + 'static>: Send + Sync + 'static {
    async fn on_event(&self, event: Event, bot: BotData<S>) -> Result<(), BotError>;
}

impl<S: Send + Sync + 'static> Bot<S> {
    pub fn new(token: impl Into<String>, intents: Intents, state: S) -> Self {
        Self {
            token: token.into(),
            intents,
            state: Arc::new(state),
            plugins: Vec::new(),
        }
    }

    pub fn add_plugin<P: Plugin<S>>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }

        let http = Arc::new(HttpClient::new(self.token.clone()));
        let application_id = http.current_user_application().await?.model().await?.id;

        let mut app = PluginBuilder {
            listeners: Vec::new(),
            commands: Vec::new(),
        };
        for p in self.plugins {
            p.build(&mut app);
        }
        if !app.commands.is_empty() {
            crate::commands::attach_dispatcher(&mut app);
        }
        let listeners = Arc::new(app.listeners);

        let bot = BotData {
            http,
            state: self.state,
            application_id,
        };

        let mut shard = Shard::new(ShardId::ONE, self.token, self.intents);

        while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
            let event = match item {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!(?e, "gateway error");
                    continue;
                }
            };
            let flag = EventTypeFlags::from(event.kind());

            let bot = bot.clone();
            let listeners = listeners.clone();
            tokio::spawn(async move {
                for entry in listeners.iter() {
                    if entry.filter.contains(flag)
                        && let Err(e) = entry.handler.on_event(event.clone(), bot.clone()).await
                    {
                        tracing::error!(?e, "listener error");
                    }
                }
            });
        }

        Ok(())
    }
}

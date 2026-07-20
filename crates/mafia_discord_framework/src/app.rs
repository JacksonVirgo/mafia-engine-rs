use crate::{
    plugin::Plugin,
    slash_command::{CommandContext, SlashCommand},
};
use std::{any::TypeId, collections::HashSet, error::Error, future::Future, pin::Pin, sync::Arc};
use twilight_gateway::{EventTypeFlags, Intents, MessageSender, Shard, ShardId, StreamExt as _};
use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::{
    event::Event,
    payload::incoming::{InteractionCreate, MessageCreate, Ready},
};

pub type BoxError = Box<dyn Error + Send + Sync>;

type HandlerFuture = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'static>>;
type EventHandler = dyn Fn(Arc<Event>, EventContext) -> HandlerFuture + Send + Sync;

pub trait EventPayload: Clone + Send + Sync + 'static {
    const EVENT_TYPE: EventTypeFlags;

    #[doc(hidden)]
    fn from_event(event: &Event) -> Option<Self>;
}

impl EventPayload for Ready {
    const EVENT_TYPE: EventTypeFlags = EventTypeFlags::READY;

    fn from_event(event: &Event) -> Option<Self> {
        match event {
            Event::Ready(payload) => Some(payload.clone()),
            _ => None,
        }
    }
}

macro_rules! boxed_event_payload {
    ($payload:ty, $variant:ident, $event_type:ident) => {
        impl EventPayload for $payload {
            const EVENT_TYPE: EventTypeFlags = EventTypeFlags::$event_type;

            fn from_event(event: &Event) -> Option<Self> {
                match event {
                    Event::$variant(payload) => Some((**payload).clone()),
                    _ => None,
                }
            }
        }
    };
}

boxed_event_payload!(MessageCreate, MessageCreate, MESSAGE_CREATE);
boxed_event_payload!(InteractionCreate, InteractionCreate, INTERACTION_CREATE);

#[derive(Clone)]
pub struct EventContext {
    http: Arc<HttpClient>,
    gateway: MessageSender,
}

impl EventContext {
    pub fn http(&self) -> &Arc<HttpClient> {
        &self.http
    }

    pub fn gateway(&self) -> &MessageSender {
        &self.gateway
    }
}

pub struct App {
    token: String,
    intents: Intents,
    wanted_events: EventTypeFlags,
    plugins: Vec<Box<dyn Plugin>>,
    plugin_types: HashSet<TypeId>,
    state: PluginState,
    middleware: Vec<Arc<EventHandler>>,
    middleware_labels: Vec<&'static str>,
    handlers: Vec<Arc<EventHandler>>,
    handler_labels: Vec<&'static str>,
    interactions: Vec<SlashCommand>,
}

#[derive(Default, PartialEq)]
enum PluginState {
    #[default]
    Adding,
    Finished,
    Cleaned,
}

impl App {
    pub fn new(token: impl Into<String>, intents: Intents) -> Self {
        Self {
            token: token.into(),
            intents,
            wanted_events: EventTypeFlags::all(),
            plugins: Vec::new(),
            plugin_types: HashSet::new(),
            state: PluginState::Adding,
            middleware: Vec::new(),
            middleware_labels: Vec::new(),
            handlers: Vec::new(),
            handler_labels: Vec::new(),
            interactions: Vec::new(),
        }
    }

    pub fn set_wanted_events(&mut self, events: EventTypeFlags) -> &mut Self {
        self.assert_adding();
        self.wanted_events = events;
        self
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        self.assert_adding();

        let id = TypeId::of::<P>();
        assert!(
            self.plugin_types.insert(id),
            "plugin {} was already added",
            std::any::type_name::<P>()
        );

        plugin.build(self);
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn add_event_handler<F, Fut>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(Arc<Event>, EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        self.add_event_handler_labeled("all events", handler)
    }

    fn add_event_handler_labeled<F, Fut>(&mut self, label: &'static str, handler: F) -> &mut Self
    where
        F: Fn(Arc<Event>, EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        self.assert_adding();
        self.handlers.push(Arc::new(move |event, context| {
            Box::pin(handler(event, context))
        }));
        self.handler_labels.push(label);
        self
    }

    pub fn add_event_middleware<F, Fut>(&mut self, middleware: F) -> &mut Self
    where
        F: Fn(Arc<Event>, EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        self.assert_adding();
        self.middleware.push(Arc::new(move |event, context| {
            Box::pin(middleware(event, context))
        }));
        self.middleware_labels.push("all events");
        self
    }

    pub fn add_event_listener<E, F, Fut>(&mut self, listener: F) -> &mut Self
    where
        E: EventPayload,
        F: Fn(E, EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        self.add_event_handler_labeled(std::any::type_name::<E>(), move |event, context| {
            let future = E::from_event(event.as_ref()).map(|payload| listener(payload, context));

            async move {
                if let Some(future) = future {
                    future.await?;
                }

                Ok(())
            }
        })
    }

    pub fn add_interaction(&mut self, command: SlashCommand) -> &mut Self {
        self.assert_adding();
        assert!(
            !self
                .interactions
                .iter()
                .any(|existing| existing.name() == command.name()),
            "slash command `{}` was already added",
            command.name()
        );
        self.interactions.push(command);
        self
    }

    pub async fn run(mut self) -> Result<(), BoxError> {
        let _ = rustls::crypto::ring::default_provider().install_default();

        while !self.plugins.iter().all(|plugin| plugin.ready(&self)) {
            tokio::task::yield_now().await;
        }

        self.call_finish();

        let http = Arc::new(HttpClient::new(self.token.clone()));
        let mut shard = Shard::new(ShardId::ONE, self.token.clone(), self.intents);

        let result = 'gateway: loop {
            let Some(item) = shard.next_event(self.wanted_events).await else {
                break Ok(());
            };
            let event = match item {
                Ok(event) => Arc::new(event),
                Err(error) => break Err(error.into()),
            };
            let context = EventContext {
                http: Arc::clone(&http),
                gateway: shard.sender(),
            };

            for middleware in &self.middleware {
                if let Err(error) = middleware(Arc::clone(&event), context.clone()).await {
                    break 'gateway Err(error);
                }
            }

            for handler in &self.handlers {
                if let Err(error) = handler(Arc::clone(&event), context.clone()).await {
                    break 'gateway Err(error);
                }
            }

            if let Err(error) = self
                .dispatch_framework_interactions(event.as_ref(), Arc::clone(&http))
                .await
            {
                break 'gateway Err(error);
            }
        };

        self.call_cleanup();
        result
    }

    async fn dispatch_framework_interactions(
        &self,
        event: &Event,
        http: Arc<HttpClient>,
    ) -> Result<(), BoxError> {
        if let Event::Ready(ready) = event {
            for command in &self.interactions {
                command.register(&http, ready.application.id).await?;
            }
        }

        let Event::InteractionCreate(payload) = event else {
            return Ok(());
        };
        let interaction = payload.0.clone();
        let Some(InteractionData::ApplicationCommand(command)) = interaction.data.clone() else {
            return Ok(());
        };
        let Some(slash_command) = self
            .interactions
            .iter()
            .find(|candidate| candidate.name() == command.name)
        else {
            return Ok(());
        };

        slash_command
            .call(CommandContext::new(http, interaction, *command))
            .await
    }

    fn assert_adding(&self) {
        assert!(
            self.state == PluginState::Adding,
            "application setup has already finished"
        );
    }

    fn call_finish(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);
        for plugin in &plugins {
            plugin.finish(self);
        }
        self.plugins = plugins;
        self.state = PluginState::Finished;

        let interactions: Vec<_> = self.interactions.iter().map(SlashCommand::name).collect();
        tracing::info!(
            intents = ?self.intents,
            wanted_events = ?self.wanted_events,
            middleware = ?self.middleware_labels,
            event_listeners = ?self.handler_labels,
            interactions = ?interactions,
            "loaded application configuration"
        );
    }

    fn call_cleanup(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);
        for plugin in &plugins {
            plugin.cleanup(self);
        }
        self.plugins = plugins;
        self.state = PluginState::Cleaned;
    }
}

use crate::plugin::Plugin;
use std::{any::TypeId, collections::HashSet, error::Error, future::Future, pin::Pin, sync::Arc};
use twilight_gateway::{EventTypeFlags, Intents, MessageSender, Shard, ShardId, StreamExt as _};
use twilight_http::Client as HttpClient;
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
    handlers: Vec<Arc<EventHandler>>,
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
            handlers: Vec::new(),
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
        self.assert_adding();
        self.handlers.push(Arc::new(move |event, context| {
            Box::pin(handler(event, context))
        }));
        self
    }

    pub fn add_event_listener<E, F, Fut>(&mut self, listener: F) -> &mut Self
    where
        E: EventPayload,
        F: Fn(E, EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        self.add_event_handler(move |event, context| {
            let future = E::from_event(event.as_ref()).map(|payload| listener(payload, context));

            async move {
                if let Some(future) = future {
                    future.await?;
                }

                Ok(())
            }
        })
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

            for handler in &self.handlers {
                if let Err(error) = handler(Arc::clone(&event), context.clone()).await {
                    break 'gateway Err(error);
                }
            }
        };

        self.call_cleanup();
        result
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

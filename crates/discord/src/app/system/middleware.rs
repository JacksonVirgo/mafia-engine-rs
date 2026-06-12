use crate::prelude::*;
use async_trait::async_trait;
use poise::serenity_prelude::{ChannelId, GuildId, UserId};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Default)]
pub struct Extensions(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

fn missing_ext<T: ?Sized>() -> BotError {
    format!(
        "missing required extension `{}`",
        std::any::type_name::<T>()
    )
    .into()
}

impl Extensions {
    pub fn insert<T: Send + Sync + 'static>(&mut self, v: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(v));
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Result<&T, BotError> {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
            .ok_or_else(|| missing_ext::<T>())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Result<&mut T, BotError> {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T>())
            .ok_or_else(|| missing_ext::<T>())
    }
}

#[derive(Clone, Debug)]
pub enum RequestKind {
    Command {
        name: String,
    },
    Component {
        custom_id: String,
        i_ctx: Option<String>,
    },
}

pub struct Request {
    pub kind: RequestKind,
    pub serenity_ctx: serenity::Context,
    pub data: BotState,
    pub user_id: UserId,
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub ext: Extensions,
}

#[derive(Clone, Debug)]
pub struct Rejection {
    pub message: String,
    pub ephemeral: bool,
}

impl Rejection {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral: true,
        }
    }

    pub fn public(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral: false,
        }
    }
}

pub enum Outcome {
    Continue,
    Reject(Rejection),
}

#[async_trait]
pub trait Middleware: Any + Send + Sync + 'static {
    async fn call(&self, req: &mut Request, next: Next<'_>) -> Result<Outcome, BotError>;
    fn requires(&self) -> Vec<DynMiddleware> {
        Vec::new()
    }
}

pub type DynMiddleware = Arc<dyn Middleware>;

pub fn expand_stack(input: Vec<DynMiddleware>) -> Vec<DynMiddleware> {
    let mut seen: HashSet<TypeId> = HashSet::new();
    let mut out: Vec<DynMiddleware> = Vec::new();
    for mw in input {
        visit_mw(mw, &mut seen, &mut out);
    }
    out
}

fn visit_mw(mw: DynMiddleware, seen: &mut HashSet<TypeId>, out: &mut Vec<DynMiddleware>) {
    let any_ref: &dyn Any = &*mw;
    let tid = any_ref.type_id();
    if !seen.insert(tid) {
        return;
    }
    for dep in mw.requires() {
        visit_mw(dep, seen, out);
    }
    out.push(mw);
}

pub struct Next<'a> {
    stack: &'a [DynMiddleware],
}

impl<'a> Next<'a> {
    pub(crate) fn new(stack: &'a [DynMiddleware]) -> Self {
        Self { stack }
    }

    pub async fn run(mut self, req: &mut Request) -> Result<Outcome, BotError> {
        match self.stack.split_first() {
            Some((head, tail)) => {
                self.stack = tail;
                head.call(req, self).await
            }
            None => Ok(Outcome::Continue),
        }
    }
}

pub async fn run_stack(stack: &[DynMiddleware], req: &mut Request) -> Result<Outcome, BotError> {
    Next::new(stack).run(req).await
}

pub trait CommandMiddlewareExt: Sized {
    fn with<M: Middleware>(self, mw: M) -> Self;
    fn with_many<I>(self, mws: I) -> Self
    where
        I: IntoIterator<Item = DynMiddleware>;
}

impl CommandMiddlewareExt for poise::Command<BotState, BotError> {
    fn with<M: Middleware>(mut self, mw: M) -> Self {
        let arc: DynMiddleware = Arc::new(mw);
        if let Some(list) = self.custom_data.downcast_mut::<Vec<DynMiddleware>>() {
            list.push(arc);
        } else {
            self.custom_data = Box::new(vec![arc]);
        }
        self
    }

    fn with_many<I>(mut self, mws: I) -> Self
    where
        I: IntoIterator<Item = DynMiddleware>,
    {
        let incoming: Vec<DynMiddleware> = mws.into_iter().collect();
        if let Some(list) = self.custom_data.downcast_mut::<Vec<DynMiddleware>>() {
            list.extend(incoming);
        } else {
            self.custom_data = Box::new(incoming);
        }
        self
    }
}

pub async fn ext<T: Clone + Send + Sync + 'static>(ctx: BotCtx<'_>) -> Result<T, BotError> {
    let guard = ctx
        .invocation_data::<Extensions>()
        .await
        .ok_or_else(|| -> BotError { "command has no middleware extensions".into() })?;
    Ok(guard.get::<T>()?.clone())
}

pub struct WithMiddleware<T> {
    pub inner: T,
    pub middleware: Vec<DynMiddleware>,
}

impl<T> WithMiddleware<T> {
    pub fn with<M: Middleware>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    pub fn with_many<I>(mut self, mws: I) -> Self
    where
        I: IntoIterator<Item = DynMiddleware>,
    {
        self.middleware.extend(mws);
        self
    }
}

pub trait ComponentMiddlewareExt: Sized {
    fn with<M: Middleware>(self, mw: M) -> WithMiddleware<Self>;
    fn with_many<I>(self, mws: I) -> WithMiddleware<Self>
    where
        I: IntoIterator<Item = DynMiddleware>;
}

impl<T> ComponentMiddlewareExt for T
where
    T: super::components::Component + Send + Sync + 'static,
{
    fn with<M: Middleware>(self, mw: M) -> WithMiddleware<Self> {
        WithMiddleware {
            inner: self,
            middleware: vec![Arc::new(mw)],
        }
    }

    fn with_many<I>(self, mws: I) -> WithMiddleware<Self>
    where
        I: IntoIterator<Item = DynMiddleware>,
    {
        WithMiddleware {
            inner: self,
            middleware: mws.into_iter().collect(),
        }
    }
}

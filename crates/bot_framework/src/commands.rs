use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::OnceCell;
use twilight_gateway::{Event, EventTypeFlags};
use twilight_model::application::command::{Command, CommandOption, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};

#[doc(hidden)]
pub mod __reexport {
    pub use twilight_model::application::command::CommandOption;
    pub use twilight_model::application::interaction::application_command::CommandDataOption;
}

use crate::app::{BotData, EventListener};
use crate::error::BotError;
use crate::plugin::PluginBuilder;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub type HandlerFn<S> = Box<
    dyn Fn(CommandCtx<S>, Vec<CommandDataOption>) -> BoxFuture<'static, Result<(), BotError>>
        + Send
        + Sync,
>;

pub struct CommandCtx<S: Send + Sync + 'static> {
    pub bot: BotData<S>,
    pub interaction: Box<Interaction>,
}

impl<S: Send + Sync + 'static> CommandCtx<S> {
    pub fn state(&self) -> &Arc<S> {
        &self.bot.state
    }

    pub async fn respond(&self, content: impl Into<String>) -> Result<(), BotError> {
        self.respond_with(content.into(), false).await
    }

    pub async fn respond_ephemeral(&self, content: impl Into<String>) -> Result<(), BotError> {
        self.respond_with(content.into(), true).await
    }

    async fn respond_with(&self, content: String, ephemeral: bool) -> Result<(), BotError> {
        let flags = ephemeral.then_some(MessageFlags::EPHEMERAL);
        let data = InteractionResponseData {
            content: Some(content),
            flags,
            ..Default::default()
        };
        let resp = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };
        self.bot
            .interaction()
            .create_response(self.interaction.id, &self.interaction.token, &resp)
            .await?;
        Ok(())
    }
}

pub trait FromInteractionOptions: Sized {
    fn from_options(options: &[CommandDataOption]) -> Result<Self, BotError>;
    fn option_specs() -> Vec<CommandOption>;
}

pub trait CommandModule<S: Send + Sync + 'static>: Sized {
    fn descriptor() -> CommandDescriptor<S>;
}

pub trait SubcommandModule<S: Send + Sync + 'static>: Sized {
    fn descriptor() -> SubcommandDescriptor<S>;
}

pub trait SubcommandGroupModule<S: Send + Sync + 'static>: Sized {
    fn descriptor() -> SubcommandGroupDescriptor<S>;
}


impl FromInteractionOptions for () {
    fn from_options(_: &[CommandDataOption]) -> Result<Self, BotError> {
        Ok(())
    }
    fn option_specs() -> Vec<CommandOption> {
        Vec::new()
    }
}

pub struct CommandDescriptor<S: Send + Sync + 'static> {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: CommandKind<S>,
}

pub enum CommandKind<S: Send + Sync + 'static> {
    Leaf(LeafHandler<S>),
    Group(Vec<SubcommandEntry<S>>),
}

pub struct LeafHandler<S: Send + Sync + 'static> {
    pub options: fn() -> Vec<CommandOption>,
    pub handler: HandlerFn<S>,
}

pub struct SubcommandDescriptor<S: Send + Sync + 'static> {
    pub name: &'static str,
    pub description: &'static str,
    pub options: fn() -> Vec<CommandOption>,
    pub handler: HandlerFn<S>,
}

pub struct SubcommandGroupDescriptor<S: Send + Sync + 'static> {
    pub name: &'static str,
    pub description: &'static str,
    pub subcommands: Vec<SubcommandDescriptor<S>>,
}

pub enum SubcommandEntry<S: Send + Sync + 'static> {
    Leaf(SubcommandDescriptor<S>),
    Group(SubcommandGroupDescriptor<S>),
}

pub trait IntoSubcommandEntry<S: Send + Sync + 'static> {
    fn into_entry(self) -> SubcommandEntry<S>;
}

impl<S: Send + Sync + 'static> IntoSubcommandEntry<S> for SubcommandDescriptor<S> {
    fn into_entry(self) -> SubcommandEntry<S> {
        SubcommandEntry::Leaf(self)
    }
}

impl<S: Send + Sync + 'static> IntoSubcommandEntry<S> for SubcommandGroupDescriptor<S> {
    fn into_entry(self) -> SubcommandEntry<S> {
        SubcommandEntry::Group(self)
    }
}

pub(crate) fn attach_dispatcher<S: Send + Sync + 'static>(app: &mut PluginBuilder<S>) {
    let commands = std::mem::take(&mut app.commands);
    let dispatcher = Arc::new(Dispatcher::new(commands));
    app.add_listener(
        EventTypeFlags::READY | EventTypeFlags::INTERACTION_CREATE,
        DispatcherListener { dispatcher },
    );
}

enum DispatchNode<S: Send + Sync + 'static> {
    Leaf {
        handler: HandlerFn<S>,
    },
    Group {
        leaves: HashMap<String, HandlerFn<S>>,
        groups: HashMap<String, HashMap<String, HandlerFn<S>>>,
    },
}

struct Dispatcher<S: Send + Sync + 'static> {
    routes: HashMap<String, DispatchNode<S>>,
    discord_commands: Vec<Command>,
    registered: OnceCell<()>,
}

impl<S: Send + Sync + 'static> Dispatcher<S> {
    fn new(descriptors: Vec<CommandDescriptor<S>>) -> Self {
        let mut routes = HashMap::new();
        let mut discord_commands = Vec::new();

        for desc in descriptors {
            discord_commands.push(to_discord_command(&desc));
            let CommandDescriptor { name, kind, .. } = desc;
            let node = match kind {
                CommandKind::Leaf(leaf) => DispatchNode::Leaf {
                    handler: leaf.handler,
                },
                CommandKind::Group(entries) => {
                    let mut leaves = HashMap::new();
                    let mut groups = HashMap::new();
                    for entry in entries {
                        match entry {
                            SubcommandEntry::Leaf(sub) => {
                                leaves.insert(sub.name.to_string(), sub.handler);
                            }
                            SubcommandEntry::Group(group) => {
                                let mut inner = HashMap::new();
                                for sub in group.subcommands {
                                    inner.insert(sub.name.to_string(), sub.handler);
                                }
                                groups.insert(group.name.to_string(), inner);
                            }
                        }
                    }
                    DispatchNode::Group { leaves, groups }
                }
            };
            routes.insert(name.to_string(), node);
        }

        Self {
            routes,
            discord_commands,
            registered: OnceCell::new(),
        }
    }

    async fn dispatch(self: Arc<Self>, ctx: CommandCtx<S>, data: CommandData) {
        let Some(node) = self.routes.get(data.name.as_str()) else {
            tracing::warn!(command = %data.name, "no handler for command");
            return;
        };

        let (handler, opts) = match node {
            DispatchNode::Leaf { handler } => (handler, data.options),
            DispatchNode::Group { leaves, groups } => {
                let Some(first) = data.options.into_iter().next() else {
                    tracing::warn!(command = %data.name, "group command missing subcommand");
                    return;
                };
                match first.value {
                    CommandOptionValue::SubCommand(opts) => {
                        let Some(h) = leaves.get(first.name.as_str()) else {
                            tracing::warn!(
                                command = %data.name,
                                sub = %first.name,
                                "no handler for subcommand",
                            );
                            return;
                        };
                        (h, opts)
                    }
                    CommandOptionValue::SubCommandGroup(inner_opts) => {
                        let Some(inner_leaves) = groups.get(first.name.as_str()) else {
                            tracing::warn!(
                                command = %data.name,
                                group = %first.name,
                                "no handler for subcommand group",
                            );
                            return;
                        };
                        let Some(inner) = inner_opts.into_iter().next() else {
                            tracing::warn!("subcommand group missing inner subcommand");
                            return;
                        };
                        let CommandOptionValue::SubCommand(opts) = inner.value else {
                            tracing::warn!("expected SubCommand inside SubCommandGroup");
                            return;
                        };
                        let Some(h) = inner_leaves.get(inner.name.as_str()) else {
                            tracing::warn!(
                                command = %data.name,
                                group = %first.name,
                                sub = %inner.name,
                                "no handler for nested subcommand",
                            );
                            return;
                        };
                        (h, opts)
                    }
                    _ => {
                        tracing::warn!("group command first option was not a subcommand");
                        return;
                    }
                }
            }
        };

        if let Err(e) = handler(ctx, opts).await {
            tracing::error!(?e, "command handler error");
        }
    }
}

fn to_discord_command<S: Send + Sync + 'static>(d: &CommandDescriptor<S>) -> Command {
    use twilight_util::builder::command::CommandBuilder;
    let mut builder = CommandBuilder::new(d.name, d.description, CommandType::ChatInput);
    match &d.kind {
        CommandKind::Leaf(leaf) => {
            for opt in (leaf.options)() {
                builder = builder.option(opt);
            }
        }
        CommandKind::Group(entries) => {
            for entry in entries {
                builder = builder.option(subcommand_entry_to_option(entry));
            }
        }
    }
    builder.build()
}

fn subcommand_entry_to_option<S: Send + Sync + 'static>(
    entry: &SubcommandEntry<S>,
) -> CommandOption {
    use twilight_model::application::command::CommandOptionType;
    match entry {
        SubcommandEntry::Leaf(sub) => CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: sub.description.to_string(),
            description_localizations: None,
            kind: CommandOptionType::SubCommand,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: sub.name.to_string(),
            name_localizations: None,
            options: Some((sub.options)()),
            required: None,
        },
        SubcommandEntry::Group(group) => CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: group.description.to_string(),
            description_localizations: None,
            kind: CommandOptionType::SubCommandGroup,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: group.name.to_string(),
            name_localizations: None,
            options: Some(
                group
                    .subcommands
                    .iter()
                    .map(|sub| CommandOption {
                        autocomplete: None,
                        channel_types: None,
                        choices: None,
                        description: sub.description.to_string(),
                        description_localizations: None,
                        kind: CommandOptionType::SubCommand,
                        max_length: None,
                        max_value: None,
                        min_length: None,
                        min_value: None,
                        name: sub.name.to_string(),
                        name_localizations: None,
                        options: Some((sub.options)()),
                        required: None,
                    })
                    .collect(),
            ),
            required: None,
        },
    }
}

struct DispatcherListener<S: Send + Sync + 'static> {
    dispatcher: Arc<Dispatcher<S>>,
}

#[async_trait]
impl<S: Send + Sync + 'static> EventListener<S> for DispatcherListener<S> {
    async fn on_event(&self, event: Event, bot: BotData<S>) -> Result<(), BotError> {
        match event {
            Event::Ready(_) => {
                let dispatcher = self.dispatcher.clone();
                let result = dispatcher
                    .registered
                    .get_or_try_init(|| async {
                        bot.interaction()
                            .set_global_commands(&dispatcher.discord_commands)
                            .await?;
                        tracing::info!(
                            count = dispatcher.discord_commands.len(),
                            "registered global commands",
                        );
                        Ok::<(), BotError>(())
                    })
                    .await;
                if let Err(e) = result {
                    tracing::error!(?e, "failed to register commands");
                }
            }
            Event::InteractionCreate(interaction) => {
                let interaction = interaction.0;
                let Some(InteractionData::ApplicationCommand(data)) = interaction.data.clone()
                else {
                    return Ok(());
                };
                let ctx = CommandCtx {
                    bot,
                    interaction: Box::new(interaction),
                };
                self.dispatcher.clone().dispatch(ctx, *data).await;
            }
            _ => {}
        }
        Ok(())
    }
}

pub mod options {
    use super::*;
    use twilight_model::application::command::CommandOptionType;

    pub fn find<'a>(opts: &'a [CommandDataOption], name: &str) -> Option<&'a CommandOptionValue> {
        opts.iter().find(|o| o.name == name).map(|o| &o.value)
    }

    pub fn required_string(opts: &[CommandDataOption], name: &str) -> Result<String, BotError> {
        match find(opts, name) {
            Some(CommandOptionValue::String(s)) => Ok(s.clone()),
            Some(_) => Err(format!("option `{name}` had wrong type").into()),
            None => Err(format!("missing required option `{name}`").into()),
        }
    }

    pub fn optional_string(
        opts: &[CommandDataOption],
        name: &str,
    ) -> Result<Option<String>, BotError> {
        match find(opts, name) {
            Some(CommandOptionValue::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(format!("option `{name}` had wrong type").into()),
            None => Ok(None),
        }
    }

    pub fn required_integer(opts: &[CommandDataOption], name: &str) -> Result<i64, BotError> {
        match find(opts, name) {
            Some(CommandOptionValue::Integer(i)) => Ok(*i),
            Some(_) => Err(format!("option `{name}` had wrong type").into()),
            None => Err(format!("missing required option `{name}`").into()),
        }
    }

    pub fn optional_integer(
        opts: &[CommandDataOption],
        name: &str,
    ) -> Result<Option<i64>, BotError> {
        match find(opts, name) {
            Some(CommandOptionValue::Integer(i)) => Ok(Some(*i)),
            Some(_) => Err(format!("option `{name}` had wrong type").into()),
            None => Ok(None),
        }
    }

    pub fn required_bool(opts: &[CommandDataOption], name: &str) -> Result<bool, BotError> {
        match find(opts, name) {
            Some(CommandOptionValue::Boolean(b)) => Ok(*b),
            Some(_) => Err(format!("option `{name}` had wrong type").into()),
            None => Err(format!("missing required option `{name}`").into()),
        }
    }

    pub fn optional_bool(opts: &[CommandDataOption], name: &str) -> Result<Option<bool>, BotError> {
        match find(opts, name) {
            Some(CommandOptionValue::Boolean(b)) => Ok(Some(*b)),
            Some(_) => Err(format!("option `{name}` had wrong type").into()),
            None => Ok(None),
        }
    }

    pub fn string_spec(name: &str, description: &str, required: bool) -> CommandOption {
        CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: description.to_string(),
            description_localizations: None,
            kind: CommandOptionType::String,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: name.to_string(),
            name_localizations: None,
            options: None,
            required: Some(required),
        }
    }

    pub fn integer_spec(name: &str, description: &str, required: bool) -> CommandOption {
        CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: description.to_string(),
            description_localizations: None,
            kind: CommandOptionType::Integer,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: name.to_string(),
            name_localizations: None,
            options: None,
            required: Some(required),
        }
    }

    pub fn bool_spec(name: &str, description: &str, required: bool) -> CommandOption {
        CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: description.to_string(),
            description_localizations: None,
            kind: CommandOptionType::Boolean,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: name.to_string(),
            name_localizations: None,
            options: None,
            required: Some(required),
        }
    }
}

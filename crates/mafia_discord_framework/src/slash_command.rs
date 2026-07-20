use crate::app::BoxError;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    future::Future,
    pin::Pin,
    sync::Arc,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::{
        command::{
            CommandOption, CommandOptionType, CommandOptionValue as CommandSchemaOptionValue,
        },
        interaction::{Interaction, application_command::CommandData},
    },
    channel::message::Embed,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{Id, marker::ApplicationMarker},
};

type CommandFuture = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'static>>;
type CommandHandler = dyn Fn(CommandContext) -> CommandFuture + Send + Sync;

pub struct SlashCommand {
    name: String,
    description: String,
    options: Vec<CommandOption>,
    handler: Option<Arc<CommandHandler>>,
}

impl SlashCommand {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            options: Vec::new(),
            handler: None,
        }
    }

    pub fn required_string(mut self, name: &str, description: &str) -> Self {
        self.options.push(Self::option(
            name,
            description,
            CommandOptionType::String,
            true,
        ));
        self
    }

    pub fn optional_string(mut self, name: &str, description: &str) -> Self {
        self.options.push(Self::option(
            name,
            description,
            CommandOptionType::String,
            false,
        ));
        self
    }

    pub fn required_integer(mut self, name: &str, description: &str) -> Self {
        self.options.push(Self::option(
            name,
            description,
            CommandOptionType::Integer,
            true,
        ));
        self
    }

    pub fn optional_integer(mut self, name: &str, description: &str) -> Self {
        self.options.push(Self::option(
            name,
            description,
            CommandOptionType::Integer,
            false,
        ));
        self
    }

    pub fn required_boolean(mut self, name: &str, description: &str) -> Self {
        self.options.push(Self::option(
            name,
            description,
            CommandOptionType::Boolean,
            true,
        ));
        self
    }

    pub fn optional_boolean(mut self, name: &str, description: &str) -> Self {
        self.options.push(Self::option(
            name,
            description,
            CommandOptionType::Boolean,
            false,
        ));
        self
    }

    pub fn optional_integer_range(
        mut self,
        name: &str,
        description: &str,
        minimum: i64,
        maximum: i64,
    ) -> Self {
        let mut option = Self::option(name, description, CommandOptionType::Integer, false);
        option.min_value = Some(CommandSchemaOptionValue::Integer(minimum));
        option.max_value = Some(CommandSchemaOptionValue::Integer(maximum));
        self.options.push(option);
        self
    }

    pub fn handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(CommandContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        self.handler = Some(Arc::new(move |context| Box::pin(handler(context))));
        self
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) async fn register(
        &self,
        http: &HttpClient,
        application_id: Id<ApplicationMarker>,
    ) -> Result<(), BoxError> {
        http.interaction(application_id)
            .create_global_command()
            .chat_input(&self.name, &self.description)
            .command_options(&self.options)
            .await?;
        Ok(())
    }

    pub(crate) async fn call(&self, context: CommandContext) -> Result<(), BoxError> {
        let handler = self
            .handler
            .as_ref()
            .ok_or_else(|| format!("slash command `{}` has no handler", self.name))?;
        handler(context).await
    }

    fn option(
        name: &str,
        description: &str,
        kind: CommandOptionType,
        required: bool,
    ) -> CommandOption {
        CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: description.into(),
            description_localizations: None,
            kind,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: name.into(),
            name_localizations: None,
            options: None,
            required: Some(required),
        }
    }
}

pub struct CommandContext {
    http: Arc<HttpClient>,
    interaction: Interaction,
    command: CommandData,
}

impl CommandContext {
    pub(crate) fn new(
        http: Arc<HttpClient>,
        interaction: Interaction,
        command: CommandData,
    ) -> Self {
        Self {
            http,
            interaction,
            command,
        }
    }

    pub fn interaction(&self) -> &Interaction {
        &self.interaction
    }

    pub fn command(&self) -> &CommandData {
        &self.command
    }

    pub fn required_string(&self, name: &str) -> Result<&str, CommandOptionError> {
        self.string(name)?
            .ok_or_else(|| CommandOptionError::missing(name, "string"))
    }

    pub fn string(&self, name: &str) -> Result<Option<&str>, CommandOptionError> {
        let Some(option) = self
            .command
            .options
            .iter()
            .find(|option| option.name == name)
        else {
            return Ok(None);
        };

        match &option.value {
            twilight_model::application::interaction::application_command::CommandOptionValue::String(value) => Ok(Some(value)),
            _ => Err(CommandOptionError::wrong_type(name, "string")),
        }
    }

    pub fn required_integer(&self, name: &str) -> Result<i64, CommandOptionError> {
        self.integer(name)?
            .ok_or_else(|| CommandOptionError::missing(name, "integer"))
    }

    pub fn integer(&self, name: &str) -> Result<Option<i64>, CommandOptionError> {
        let Some(option) = self
            .command
            .options
            .iter()
            .find(|option| option.name == name)
        else {
            return Ok(None);
        };

        match option.value {
            twilight_model::application::interaction::application_command::CommandOptionValue::Integer(value) => Ok(Some(value)),
            _ => Err(CommandOptionError::wrong_type(name, "integer")),
        }
    }

    pub fn required_boolean(&self, name: &str) -> Result<bool, CommandOptionError> {
        self.boolean(name)?
            .ok_or_else(|| CommandOptionError::missing(name, "boolean"))
    }

    pub fn boolean(&self, name: &str) -> Result<Option<bool>, CommandOptionError> {
        let Some(option) = self
            .command
            .options
            .iter()
            .find(|option| option.name == name)
        else {
            return Ok(None);
        };

        match option.value {
            twilight_model::application::interaction::application_command::CommandOptionValue::Boolean(value) => Ok(Some(value)),
            _ => Err(CommandOptionError::wrong_type(name, "boolean")),
        }
    }

    pub async fn respond(&self, content: impl Into<String>) -> Result<(), BoxError> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(content.into()),
                ..InteractionResponseData::default()
            }),
        };

        self.http
            .interaction(self.interaction.application_id)
            .create_response(self.interaction.id, &self.interaction.token, &response)
            .await?;
        Ok(())
    }

    pub async fn respond_embed(&self, embed: Embed) -> Result<(), BoxError> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                embeds: Some(vec![embed]),
                ..InteractionResponseData::default()
            }),
        };

        self.http
            .interaction(self.interaction.application_id)
            .create_response(self.interaction.id, &self.interaction.token, &response)
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CommandOptionError {
    name: String,
    expected: &'static str,
    reason: &'static str,
}

impl CommandOptionError {
    fn missing(name: &str, expected: &'static str) -> Self {
        Self {
            name: name.into(),
            expected,
            reason: "is missing",
        }
    }

    fn wrong_type(name: &str, expected: &'static str) -> Self {
        Self {
            name: name.into(),
            expected,
            reason: "has a different type",
        }
    }
}

impl Display for CommandOptionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "command option `{}` {} (expected {})",
            self.name, self.reason, self.expected
        )
    }
}

impl Error for CommandOptionError {}

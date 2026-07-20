use crate::{app::BoxError, context::GlobalContext};
use std::{future::Future, pin::Pin, sync::Arc};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::{Interaction, message_component::MessageComponentInteractionData},
    channel::message::{
        Embed,
        component::{
            ActionRow, Button as TwilightButton, ButtonStyle, Component as TwilightComponent,
            SelectMenu, SelectMenuOption, SelectMenuType,
        },
    },
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

type ComponentFuture = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'static>>;
type ComponentCallback = dyn Fn(ComponentContext) -> ComponentFuture + Send + Sync;

/// A raw Discord message component used when constructing an interaction response.
pub type DiscordComponent = TwilightComponent;

/// The Discord payload supplied when a struct-backed [`Button`] is pressed.
pub type ButtonData = MessageComponentInteractionData;
/// The Discord payload supplied when a struct-backed [`TextSelect`] is submitted.
pub type TextSelectData = MessageComponentInteractionData;
/// The Discord payload supplied when a struct-backed [`UserSelect`] is submitted.
pub type UserSelectData = MessageComponentInteractionData;
/// The Discord payload supplied when a struct-backed [`RoleSelect`] is submitted.
pub type RoleSelectData = MessageComponentInteractionData;
/// The Discord payload supplied when a struct-backed [`MentionableSelect`] is submitted.
pub type MentionableSelectData = MessageComponentInteractionData;
/// The Discord payload supplied when a struct-backed [`ChannelSelect`] is submitted.
pub type ChannelSelectData = MessageComponentInteractionData;

pub struct ComponentContext {
    http: Arc<HttpClient>,
    interaction: Interaction,
    data: MessageComponentInteractionData,
    global: GlobalContext,
}

impl ComponentContext {
    pub(crate) fn new(
        http: Arc<HttpClient>,
        interaction: Interaction,
        data: MessageComponentInteractionData,
        global: GlobalContext,
    ) -> Self {
        Self {
            http,
            interaction,
            data,
            global,
        }
    }

    pub fn interaction(&self) -> &Interaction {
        &self.interaction
    }

    pub fn data(&self) -> &MessageComponentInteractionData {
        &self.data
    }

    pub fn custom_id(&self) -> &str {
        &self.data.custom_id
    }

    pub fn values(&self) -> &[String] {
        &self.data.values
    }

    pub fn global_context(&self) -> &GlobalContext {
        &self.global
    }

    pub fn global<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.global.get()
    }

    pub async fn respond_empty(&self) -> Result<(), BoxError> {
        self.respond_with(InteractionResponseData::default()).await
    }

    pub async fn respond(&self, content: impl Into<String>) -> Result<(), BoxError> {
        self.respond_with(InteractionResponseData {
            content: Some(content.into()),
            ..InteractionResponseData::default()
        })
        .await
    }

    pub async fn respond_components(
        &self,
        content: impl Into<String>,
        components: Vec<DiscordComponent>,
    ) -> Result<(), BoxError> {
        self.respond_with(InteractionResponseData {
            content: Some(content.into()),
            components: Some(components),
            ..InteractionResponseData::default()
        })
        .await
    }

    pub async fn respond_embed(&self, embed: Embed) -> Result<(), BoxError> {
        self.respond_with(InteractionResponseData {
            embeds: Some(vec![embed]),
            ..InteractionResponseData::default()
        })
        .await
    }

    async fn respond_with(&self, data: InteractionResponseData) -> Result<(), BoxError> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        self.http
            .interaction(self.interaction.application_id)
            .create_response(self.interaction.id, &self.interaction.token, &response)
            .await?;
        Ok(())
    }
}

/// A struct-backed button. Override only the methods the button needs.
///
/// `build` defaults to an empty secondary button, and `run` acknowledges the interaction with an
/// empty response. `run` receives [`ButtonData`] separately, so the button payload does not need
/// to be read through [`ComponentContext`].
pub trait Button: Send + Sync + 'static {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::empty_button()
    }

    fn run(
        &self,
        context: ComponentContext,
        _data: ButtonData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        async move { context.respond_empty().await }
    }
}

macro_rules! select_component {
    ($doc:literal, $trait_name:ident, $data_name:ident, $empty_builder:ident) => {
        #[doc = $doc]
        pub trait $trait_name: Send + Sync + 'static {
            fn build(&self) -> ComponentBuilder {
                ComponentBuilder::$empty_builder()
            }

            fn run(
                &self,
                context: ComponentContext,
                _data: $data_name,
            ) -> impl Future<Output = Result<(), BoxError>> + Send {
                async move { context.respond_empty().await }
            }
        }
    };
}

select_component!(
    "A struct-backed text select menu.",
    TextSelect,
    TextSelectData,
    empty_text_select
);
select_component!(
    "A struct-backed user select menu.",
    UserSelect,
    UserSelectData,
    empty_user_select
);
select_component!(
    "A struct-backed role select menu.",
    RoleSelect,
    RoleSelectData,
    empty_role_select
);
select_component!(
    "A struct-backed mentionable select menu.",
    MentionableSelect,
    MentionableSelectData,
    empty_mentionable_select
);
select_component!(
    "A struct-backed channel select menu.",
    ChannelSelect,
    ChannelSelectData,
    empty_channel_select
);

pub struct RegisteredComponent {
    custom_id: String,
    component: DiscordComponent,
    callback: Arc<ComponentCallback>,
}

impl RegisteredComponent {
    pub fn component(&self) -> &DiscordComponent {
        &self.component
    }

    pub(crate) fn custom_id(&self) -> &str {
        &self.custom_id
    }

    pub(crate) async fn call(&self, context: ComponentContext) -> Result<(), BoxError> {
        (self.callback)(context).await
    }
}

/// A struct-backed component ready to be registered with [`crate::App::add_component`].
pub struct Component {
    registered: RegisteredComponent,
}

impl Component {
    pub fn button<B>(button: B) -> Self
    where
        B: Button,
    {
        let button = Arc::new(button);
        let registered = button.build().build(move |context| {
            let button = Arc::clone(&button);
            let data = context.data().clone();
            async move { button.run(context, data).await }
        });
        Self { registered }
    }

    pub fn text_select<S>(select: S) -> Self
    where
        S: TextSelect,
    {
        let select = Arc::new(select);
        let registered = select.build().build(move |context| {
            let select = Arc::clone(&select);
            let data = context.data().clone();
            async move { select.run(context, data).await }
        });
        Self { registered }
    }

    pub fn user_select<S>(select: S) -> Self
    where
        S: UserSelect,
    {
        let select = Arc::new(select);
        let registered = select.build().build(move |context| {
            let select = Arc::clone(&select);
            let data = context.data().clone();
            async move { select.run(context, data).await }
        });
        Self { registered }
    }

    pub fn role_select<S>(select: S) -> Self
    where
        S: RoleSelect,
    {
        let select = Arc::new(select);
        let registered = select.build().build(move |context| {
            let select = Arc::clone(&select);
            let data = context.data().clone();
            async move { select.run(context, data).await }
        });
        Self { registered }
    }

    pub fn mentionable_select<S>(select: S) -> Self
    where
        S: MentionableSelect,
    {
        let select = Arc::new(select);
        let registered = select.build().build(move |context| {
            let select = Arc::clone(&select);
            let data = context.data().clone();
            async move { select.run(context, data).await }
        });
        Self { registered }
    }

    pub fn channel_select<S>(select: S) -> Self
    where
        S: ChannelSelect,
    {
        let select = Arc::new(select);
        let registered = select.build().build(move |context| {
            let select = Arc::clone(&select);
            let data = context.data().clone();
            async move { select.run(context, data).await }
        });
        Self { registered }
    }

    pub(crate) fn into_registered(self) -> RegisteredComponent {
        self.registered
    }
}

pub struct ComponentBuilder {
    custom_id: String,
    component: DiscordComponent,
}

impl ComponentBuilder {
    pub fn empty_button() -> Self {
        Self::button("", "", ButtonStyle::Secondary)
    }

    pub fn empty_text_select() -> Self {
        Self::text_select("")
    }

    pub fn empty_user_select() -> Self {
        Self::user_select("")
    }

    pub fn empty_role_select() -> Self {
        Self::role_select("")
    }

    pub fn empty_mentionable_select() -> Self {
        Self::mentionable_select("")
    }

    pub fn empty_channel_select() -> Self {
        Self::channel_select("")
    }

    pub fn component(&self) -> &DiscordComponent {
        &self.component
    }

    pub fn button(
        custom_id: impl Into<String>,
        label: impl Into<String>,
        style: ButtonStyle,
    ) -> Self {
        let custom_id = custom_id.into();
        Self {
            component: DiscordComponent::Button(TwilightButton {
                id: None,
                custom_id: Some(custom_id.clone()),
                disabled: false,
                emoji: None,
                label: Some(label.into()),
                style,
                url: None,
                sku_id: None,
            }),
            custom_id,
        }
    }

    pub fn text_select(custom_id: impl Into<String>) -> Self {
        Self::select_menu(custom_id, SelectMenuType::Text)
    }

    pub fn user_select(custom_id: impl Into<String>) -> Self {
        Self::select_menu(custom_id, SelectMenuType::User)
    }

    pub fn role_select(custom_id: impl Into<String>) -> Self {
        Self::select_menu(custom_id, SelectMenuType::Role)
    }

    pub fn mentionable_select(custom_id: impl Into<String>) -> Self {
        Self::select_menu(custom_id, SelectMenuType::Mentionable)
    }

    pub fn channel_select(custom_id: impl Into<String>) -> Self {
        Self::select_menu(custom_id, SelectMenuType::Channel)
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        if let DiscordComponent::SelectMenu(select_menu) = &mut self.component {
            select_menu.placeholder = Some(placeholder.into());
        }
        self
    }

    pub fn options(mut self, options: Vec<SelectMenuOption>) -> Self {
        if let DiscordComponent::SelectMenu(select_menu) = &mut self.component {
            select_menu.options = Some(options);
        }
        self
    }

    pub fn build<F, Fut>(self, callback: F) -> RegisteredComponent
    where
        F: Fn(ComponentContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        RegisteredComponent {
            custom_id: self.custom_id,
            component: self.component,
            callback: Arc::new(move |context| Box::pin(callback(context))),
        }
    }

    fn select_menu(custom_id: impl Into<String>, kind: SelectMenuType) -> Self {
        let custom_id = custom_id.into();
        Self {
            component: DiscordComponent::SelectMenu(SelectMenu {
                id: None,
                channel_types: None,
                custom_id: custom_id.clone(),
                default_values: None,
                disabled: false,
                kind,
                max_values: None,
                min_values: None,
                options: None,
                placeholder: None,
                required: None,
            }),
            custom_id,
        }
    }
}

pub struct ActionRowBuilder {
    components: Vec<DiscordComponent>,
}

impl ActionRowBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn component(mut self, component: impl Into<DiscordComponent>) -> Self {
        self.components.push(component.into());
        self
    }

    pub fn build(self) -> DiscordComponent {
        DiscordComponent::ActionRow(ActionRow {
            id: None,
            components: self.components,
        })
    }
}

impl Default for ActionRowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn select_option(label: impl Into<String>, value: impl Into<String>) -> SelectMenuOption {
    SelectMenuOption {
        default: false,
        description: None,
        emoji: None,
        label: label.into(),
        value: value.into(),
    }
}

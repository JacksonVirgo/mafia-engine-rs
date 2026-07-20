use mafia_discord_framework::prelude::*;
use std::future::Future;

pub fn register(app: &mut App) {
    let test_components = vec![
        ActionRowBuilder::new()
            .component(PrimaryButton.build().component().clone())
            .component(MoreButton.build().component().clone())
            .component(SuccessButton.build().component().clone())
            .component(DangerButton.build().component().clone())
            .build(),
        ActionRowBuilder::new()
            .component(TextMenu.build().component().clone())
            .build(),
        ActionRowBuilder::new()
            .component(UserMenu.build().component().clone())
            .build(),
        ActionRowBuilder::new()
            .component(RoleMenu.build().component().clone())
            .build(),
        ActionRowBuilder::new()
            .component(MentionableMenu.build().component().clone())
            .build(),
    ];

    app.add_component(Component::button(PrimaryButton))
        .add_component(Component::button(MoreButton))
        .add_component(Component::button(SuccessButton))
        .add_component(Component::button(DangerButton))
        .add_component(Component::text_select(TextMenu))
        .add_component(Component::user_select(UserMenu))
        .add_component(Component::role_select(RoleMenu))
        .add_component(Component::mentionable_select(MentionableMenu))
        .add_component(Component::channel_select(ChannelMenu))
        .add_interaction(command(test_components));
}

fn command(components: Vec<DiscordComponent>) -> SlashCommand {
    SlashCommand::new("test", "Show interactive component examples").handler(move |context| {
        let components = components.clone();

        async move {
            context
                .respond_components("Interactive component examples", components)
                .await
        }
    })
}

struct PrimaryButton;

impl Button for PrimaryButton {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::button("test:primary", "Primary", ButtonStyle::Primary)
    }

    fn run(
        &self,
        context: ComponentContext,
        data: ButtonData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

struct MoreButton;

impl Button for MoreButton {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::button("test:more", "Show channel select", ButtonStyle::Secondary)
    }

    fn run(
        &self,
        context: ComponentContext,
        _data: ButtonData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        async move {
            let components = vec![
                ActionRowBuilder::new()
                    .component(ChannelMenu.build().component().clone())
                    .build(),
            ];
            context
                .respond_components("Channel select component", components)
                .await
        }
    }
}

struct SuccessButton;

impl Button for SuccessButton {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::button("test:success", "Success", ButtonStyle::Success)
    }

    fn run(
        &self,
        context: ComponentContext,
        data: ButtonData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

struct DangerButton;

impl Button for DangerButton {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::button("test:danger", "Danger", ButtonStyle::Danger)
    }

    fn run(
        &self,
        context: ComponentContext,
        data: ButtonData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

struct TextMenu;

impl TextSelect for TextMenu {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::text_select("test:text-select")
            .placeholder("Choose a text option")
            .options(vec![
                select_option("Alpha", "alpha"),
                select_option("Beta", "beta"),
            ])
    }

    fn run(
        &self,
        context: ComponentContext,
        data: TextSelectData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

struct UserMenu;

impl UserSelect for UserMenu {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::user_select("test:user-select").placeholder("Choose a user")
    }

    fn run(
        &self,
        context: ComponentContext,
        data: UserSelectData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

struct RoleMenu;

impl RoleSelect for RoleMenu {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::role_select("test:role-select").placeholder("Choose a role")
    }

    fn run(
        &self,
        context: ComponentContext,
        data: RoleSelectData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

struct MentionableMenu;

impl MentionableSelect for MentionableMenu {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::mentionable_select("test:mentionable-select")
            .placeholder("Choose a user or role")
    }

    fn run(
        &self,
        context: ComponentContext,
        data: MentionableSelectData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

struct ChannelMenu;

impl ChannelSelect for ChannelMenu {
    fn build(&self) -> ComponentBuilder {
        ComponentBuilder::channel_select("test:channel-select").placeholder("Choose a channel")
    }

    fn run(
        &self,
        context: ComponentContext,
        data: ChannelSelectData,
    ) -> impl Future<Output = Result<(), BoxError>> + Send {
        acknowledge(context, data)
    }
}

async fn acknowledge(context: ComponentContext, data: ButtonData) -> Result<(), BoxError> {
    let values = if data.values.is_empty() {
        "no selected values".to_owned()
    } else {
        data.values.join(", ")
    };

    context
        .respond(format!("Handled `{}` with {values}.", data.custom_id))
        .await
}

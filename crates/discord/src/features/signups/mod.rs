pub mod buttons;
pub mod create;
pub mod embed;
pub mod manage;

use crate::{
    features::middleware::{
        require_manage_channels::RequireManageChannels, require_server::RequireServer,
    },
    prelude::*,
};
use buttons::{
    join::SignupJoin, leave::SignupLeave, refresh::SignupRefresh, settings::SignupSettings,
};
use create::create;
use manage::{
    add_user::SignupAddUserMenu, category_select::SignupCategorySelect, cull::SignupCullButton,
    home::SignupHomeButton, playerchats::SignupPlayerChatsButton,
    remove_user::SignupRemoveUserMenu,
};

#[poise::command(slash_command, subcommands("create"), subcommand_required)]
pub async fn signups(_: BotCtx<'_>) -> Result<(), BotError> {
    Ok(())
}

plugin!(SignupPlugin, |app| {
    let mut cmd = signups();
    cmd.subcommands = vec![create::create().with(RequireServer)];
    app.add_command(cmd);

    app.add_component(SignupRefresh);
    app.add_component(SignupLeave);
    app.add_component(SignupJoin);

    app.add_component(SignupSettings.with(RequireManageChannels));
    app.add_component(SignupHomeButton);
    app.add_component(SignupPlayerChatsButton);
    app.add_component(SignupCategorySelect);
    app.add_component(SignupCullButton);
    app.add_component(SignupAddUserMenu);
    app.add_component(SignupRemoveUserMenu);
});

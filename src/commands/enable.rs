use serenity::{
    all::{CommandInteraction, Permissions},
    builder::CreateCommand,
};

use crate::data::AppData;

pub fn run(app_data: &AppData, command: &CommandInteraction) -> String {
    let Some(guild_id) = command.guild_id else {
        return "No server ID found, unable to enable auto scanning".to_string();
    };

    if app_data.enable_auto_scan(&guild_id).is_err() {
        return "Failed updating the database, unable to enable auto scanning".to_string();
    }

    return "Automatic role scanning is now active".to_string();
}

pub fn register() -> CreateCommand {
    CreateCommand::new("enable")
        .description("Enables the automatic role scanner")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

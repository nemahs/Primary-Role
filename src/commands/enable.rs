use serenity::{
    all::{CommandInteraction, Permissions},
    builder::CreateCommand,
};

use crate::data::AppData;

/// Enable automatic role scanning for the given server.
///
/// @param app_data Database to update
/// @param command Discord command to process
///
/// @return Result message to display to the user
pub fn run(app_data: &AppData, command: &CommandInteraction) -> String {
    let Some(guild_id) = command.guild_id else {
        return "No server ID found, unable to enable auto scanning".to_string();
    };

    if app_data.enable_auto_scan(&guild_id).is_err() {
        return "Failed updating the database, unable to enable auto scanning".to_string();
    }

    return "Automatic role scanning is now active".to_string();
}

/// Create the command to register with Discord.
pub fn register() -> CreateCommand {
    CreateCommand::new("enable")
        .description("Enables the automatic role scanner")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

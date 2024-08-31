use serenity::all::{CommandInteraction, Permissions};
use serenity::builder::CreateCommand;

use crate::data::AppData;

/// Disable the automatic role scanning for the given server
///
/// @param data Database to update
/// @param command Discord command to process
///
/// @return Result message to display to the user
pub fn run(data: &AppData, command: &CommandInteraction) -> String {
    let Some(guild_id) = command.guild_id else {
        return "No server ID given, unable to disable auto scanning".to_string();
    };

    if data.disable_auto_scan(&guild_id).is_err() {
        return "A database error occurred, unable to disable auto scanning".to_string();
    }

    return "Automatic Role Scanning is no longer active".to_string();
}

/// Create the command to register to Discord
pub fn register() -> CreateCommand {
    CreateCommand::new("disable")
        .description("Disables the automatic role scanner")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

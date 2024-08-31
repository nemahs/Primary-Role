use serenity::all::{CommandInteraction, Permissions};
use serenity::builder::CreateCommand;

use crate::data::AppData;

pub fn run(data: &AppData, interaction: &CommandInteraction) -> String {
    let Some(guild_id) = interaction.guild_id else {
        return "No server ID given, unable to disable auto scanning".to_string();
    };

    if data.disable_auto_scan(&guild_id).is_err() {
        return "A database error occurred, unable to disable auto scanning".to_string();
    }

    return "Automatic Role Scanning is no longer active".to_string();
}

pub fn register() -> CreateCommand {
    CreateCommand::new("disable")
        .description("Disables the automatic role scanner")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

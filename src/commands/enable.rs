use serenity::{
    all::{CommandInteraction, Permissions},
    builder::CreateCommand,
};

use crate::data::AppData;

pub fn run(app_data: &AppData, command: &CommandInteraction) -> String {
    app_data
        .enable_auto_scan(&command.guild_id.unwrap())
        .unwrap();
    "Automatic role scanning is now active".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("enable")
        .description("Enables the automatic role scanner")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

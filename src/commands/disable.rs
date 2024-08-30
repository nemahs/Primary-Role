use serenity::all::{CommandInteraction, Permissions};
use serenity::builder::CreateCommand;

use crate::data::AppData;

pub fn run(data: &AppData, interaction: &CommandInteraction) -> String {
    data.disable_auto_scan(&interaction.guild_id.unwrap())
        .unwrap();
    "Automatic Role Scanning is no longer active".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("disable")
        .description("Disables the automatic role scanner")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

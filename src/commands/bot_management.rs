use serenity::all::*;

use crate::{commands::commands::DiscordCommand, data::AppData};

pub struct ScanningCommands;

impl ScanningCommands{

    async fn enable(command: &CommandInteraction, app_data: &mut AppData) -> String {
        let Some(guild_id) = command.guild_id else {
            return "No server ID found, unable to enable auto scanning".to_string();
        };

        if app_data.enable_auto_scan(&guild_id).is_err() {
            return "Failed updating the database, unable to enable auto scanning".to_string();
        }

        return "Automatic role scanning is now active".to_string();
    }

    async fn disable(command: &CommandInteraction, data: &mut AppData) -> String {
        let Some(guild_id) = command.guild_id else {
            return "No server ID given, unable to disable auto scanning".to_string();
        };

        if data.disable_auto_scan(&guild_id).is_err() {
            return "A database error occurred, unable to disable auto scanning".to_string();
        }

        return "Automatic Role Scanning is no longer active".to_string();
    }

    async fn status(command: &CommandInteraction, data: &mut AppData) -> String {
        let Some(guild_id) = command.guild_id else {
            return "No server ID found, unable to check status".to_string();
        };

        let is_enabled = data.is_auto_scan_enabled(&guild_id);
        return format!("Automatic role scanning is currently {}", if is_enabled { "enabled" } else { "disabled" }).to_string();
    }
}

#[async_trait]
impl DiscordCommand for ScanningCommands {
    
    async fn run(&self, _ctx: &Context, command: &CommandInteraction, data: &mut AppData) -> String {

        let Some(subcommand) = command.data.options.get(0) else {
            return "No subcommand given".to_string();
        };

        match subcommand.name.as_str() {
            "enable" => ScanningCommands::enable(command, data).await,
            "disable" => ScanningCommands::disable(command, data).await,
            "status" => ScanningCommands::status(command, data).await,
            _ => "Unknown subcommand".to_string(),
        }
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("scanning")
            .description("Commands to manage the automatic role scanner")
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "enable", "Enable the automatic role scanner"),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "disable", "Disable the automatic role scanner"),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "status", "Check if the automatic role scanner is enabled or disabled"),
            )
            .add_context(InteractionContext::Guild)
    }
}
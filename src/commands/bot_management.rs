use serenity::all::*;

use crate::{commands::commands::DiscordCommand, data::AppData};

pub struct EnableCommand;

#[async_trait]
impl DiscordCommand for EnableCommand {
    /// Enable automatic role scanning for the given server.
    ///
    /// @param app_data Database to update
    /// @param command Discord command to process
    ///
    /// @return Result message to display to the user
    async fn run(&self, _ctx: &Context, command: &CommandInteraction, app_data: &mut AppData) -> String {
        let Some(guild_id) = command.guild_id else {
            return "No server ID found, unable to enable auto scanning".to_string();
        };

        if app_data.enable_auto_scan(&guild_id).is_err() {
            return "Failed updating the database, unable to enable auto scanning".to_string();
        }

        return "Automatic role scanning is now active".to_string();
    }

    /// Create the command to register with Discord.
    fn register(&self) -> CreateCommand {
        CreateCommand::new("enable")
            .description("Enables the automatic role scanner")
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }
}

pub struct DisableCommand;

#[async_trait]
impl DiscordCommand for DisableCommand {
    /// Disable the automatic role scanning for the given server
    ///
    /// @param data Database to update
    /// @param command Discord command to process
    ///
    /// @return Result message to display to the user
    async fn run(&self, _ctx: &Context, command: &CommandInteraction, data: &mut AppData) -> String {
        let Some(guild_id) = command.guild_id else {
            return "No server ID given, unable to disable auto scanning".to_string();
        };

        if data.disable_auto_scan(&guild_id).is_err() {
            return "A database error occurred, unable to disable auto scanning".to_string();
        }

        return "Automatic Role Scanning is no longer active".to_string();
    }

    /// Create the command to register to Discord
    fn register(&self) -> CreateCommand {
        CreateCommand::new("disable")
            .description("Disables the automatic role scanner")
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }
}

pub struct StatusCommand;

#[async_trait]
impl DiscordCommand for StatusCommand {
    /// Check if automatic role scanning is enabled for this server
    ///
    /// @param command Discord command to process
    /// @param data Database to query
    /// @param ctx Context object for the command being processed
    ///
    /// @return Result message to display to the user
    async fn run(&self, _ctx: &Context, command: &CommandInteraction, data: &mut AppData) -> String {
        let Some(guild_id) = command.guild_id else {
            return "No server ID found, unable to check status".to_string();
        };

        let is_enabled = data.is_auto_scan_enabled(&guild_id);
        return format!("Automatic role scanning is currently {}", if is_enabled { "enabled" } else { "disabled" }).to_string();
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("status")
            .description("Check if the automatic role scanner is enabled or disabled")
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }
}

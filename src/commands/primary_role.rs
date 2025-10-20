use serenity::all::*;

use crate::{
    commands::commands::{get_option, DiscordCommand},
    data::AppData,
};

pub struct PrimaryRoleCommands;

impl PrimaryRoleCommands {
    async fn set(ctx: &Context, guild_id: Option<GuildId>, command: &CommandDataOptionValue, data: &mut AppData) -> String {
        let options = if let CommandDataOptionValue::SubCommand(options) = command {
            options
        } else {
            return "Invalid command data".to_string();
        };

        let Some(new_id) = get_option("role_id", &options) else {
            return "No role ID given".to_string();
        };
        let Some(new_id) = new_id.value.as_role_id() else {
            return "Given role ID is invalid".to_string();
        };
        let Some(guild_id) = guild_id else {
            return "No server ID found".to_string();
        };

        // Validate role exists
        let Ok(roles) = guild_id.roles(&ctx).await else {
            return "Failed to get list of roles from the server".to_string();
        };

        if !roles.contains_key(&new_id) {
            return "Given role is not in this server".to_string();
        }
        // Update database
        if data.update_server_primary_role(&guild_id, &new_id).is_err() {
            return "Failed to update primary role in the database".to_string();
        }

        return format!("Updated primary role to {}", new_id.get()).to_string();
    }

    async fn get(command: &CommandInteraction, data: &mut AppData) -> String {
        let options = &command.data;
        let Some(guild_id) = options.guild_id else {
            return "No server ID found".to_string();
        };

        let primary_role = data.get_primary_role(&guild_id);
        let Some(primary_role) = primary_role else {
            return "No primary role set for this server".to_string();
        };

        return format!("The primary role for this server is {}", primary_role.get()).to_string();
    }
}

#[async_trait]
impl DiscordCommand for PrimaryRoleCommands {
    async fn run(&self, ctx: &Context, command: &CommandInteraction, data: &mut AppData) -> String {
        let Some(subcommand) = command.data.options.get(0) else {
            return "No subcommand given".to_string();
        };

        match subcommand.name.as_str() {
            "set" => PrimaryRoleCommands::set(ctx, command.guild_id, &subcommand.value, data).await,
            "get" => PrimaryRoleCommands::get(command, data).await,
            _ => "Unknown subcommand".to_string(),
        }
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("primaryrole")
            .description("Commands to manage the primary role for this server")
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "set", "Set the primary role for this server")
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::Role, "role_id", "Role to become the new primary role").required(true)),
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "get",
                "Get the current primary role for this server",
            ))
    }
}

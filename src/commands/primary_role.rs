use serenity::all::*;

use crate::{
    commands::commands::{get_option, DiscordCommand},
    data::AppData,
};

pub struct SetPrimaryRoleCommand;

#[async_trait]
impl DiscordCommand for SetPrimaryRoleCommand {
    /// Set the primary role for a given server that users must have in order to hold other roles
    ///
    /// @param ctx Context object for the command being processed
    /// @param options Options provided in the command
    /// @param data Database to update
    ///
    /// @return Result message to display to the user
    async fn run(&self, ctx: &Context, command: &CommandInteraction, data: &mut AppData) -> String {
        let options = &command.data;

        let Some(new_id) = get_option("role_id", &options.options) else {
            return "No role ID given".to_string();
        };
        let Some(new_id) = new_id.value.as_role_id() else {
            return "Given role ID is invalid".to_string();
        };
        let Some(guild_id) = options.guild_id else {
            return "No server ID found".to_string();
        };

        // Validate role exists
        let Ok(roles) = guild_id.roles(&ctx.http).await else {
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

    /// Create the command to register with Discord
    fn register(&self) -> CreateCommand {
        let id_option = CreateCommandOption::new(CommandOptionType::Role, "role_id", "Role to become the new primary role").required(true);

        CreateCommand::new("setprimaryrole")
            .description("Set the primary role for this server that all members must have in order to be given another role.")
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .add_option(id_option)
    }
}

pub struct GetPrimaryRoleCommand;

#[async_trait]
impl DiscordCommand for GetPrimaryRoleCommand {
    async fn run(&self, _: &Context, command: &CommandInteraction, data: &mut AppData) -> String {
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

    /// Create the command to register with Discord
    fn register(&self) -> CreateCommand {
        CreateCommand::new("getprimaryrole")
            .description("Get the primary role for this server that all members must have in order to be given another role.")
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }
}

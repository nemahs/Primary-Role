use serenity::all::*;

use crate::data::AppData;

/// Retrieve a given option from the list of provided options
///
/// @param name Option name to find
/// @param vec List of provided command options
///
/// @return Option matching the given name, or None if not found
fn get_option(name: &str, vec: &Vec<CommandDataOption>) -> Option<CommandDataOption> {
    for option in vec {
        if option.name == name {
            return Some(option.clone());
        }
    }

    return None;
}

/// Set the primary role for a given server that users must have in order to hold other roles
///
/// @param ctx Context object for the command being processed
/// @param options Options provided in the command
/// @param data Database to update
///
/// @return Result message to display to the user
pub async fn run(ctx: &Context, options: &CommandData, data: &mut AppData) -> String {
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
pub fn register() -> CreateCommand {
    let id_option = CreateCommandOption::new(
        CommandOptionType::Role,
        "role_id",
        "Role to become the new primary role",
    )
    .required(true);

    CreateCommand::new("primaryrole")
    .description("Set the primary role for this server that all members must have in order to be given another role.")
    .default_member_permissions(Permissions::ADMINISTRATOR)
    .add_option(id_option)
}

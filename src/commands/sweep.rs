use serenity::{
    all::{CommandInteraction, Context, Permissions},
    builder::CreateCommand,
};

use crate::data::AppData;
use tokio::sync::Mutex;

pub async fn run(ctx: &Context, command: &CommandInteraction, app_data: &Mutex<AppData>) -> String {
    let Some(guild_id) = command.guild_id else {
        return "No server ID was given".to_string();
    };

    let mut total_members: u64 = 0;
    let mut removed_roles: u64 = 0;

    let app_data = app_data.lock().await;
    let Some(primary_role) = app_data.get_primary_role(&guild_id) else {
        return "Failed to determine the primary role for this server".to_string();
    };

    let Ok(member_list) = guild_id.members(&ctx.http, None, None).await else {
        return "Failed to get the member list for the server".to_string();
    };

    for member in member_list {
        total_members += 1;
        if !member.roles.contains(&primary_role) && !member.user.bot && !member.roles.is_empty() {
            let result = member.remove_roles(&ctx.http, &member.roles).await;
            removed_roles += 1;

            if result.is_err() {
                return format!("Failed to remove roles from {}", member.display_name())
                    .to_string();
            }
        }
    }

    return format!("Removed roles on {removed_roles} members of {total_members} in server.")
        .to_string();
}

pub fn register() -> CreateCommand {
    CreateCommand::new("sweep")
        .description(
            "Sweep the current server and remove roles from members without the mandatory role.",
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

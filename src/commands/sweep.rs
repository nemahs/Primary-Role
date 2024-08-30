use serenity::{
    all::{CommandInteraction, Context, Permissions},
    builder::CreateCommand,
};

use crate::data::AppData;
use tokio::sync::Mutex;

pub async fn run(ctx: &Context, command: &CommandInteraction, app_data: &Mutex<AppData>) -> String {
    if let Some(guild_id) = command.guild_id {
        let mut total_members = 0;
        let mut removed_roles = 0;

        let app_data = app_data.lock().await;
        if let Some(primary_role) = app_data.get_primary_role(&guild_id) {
            let member_list = guild_id.members(&ctx.http, None, None).await;

            match member_list {
                Ok(member_list) => {
                    for member in member_list {
                        total_members += 1;
                        if !member.roles.contains(&primary_role)
                            && !member.user.bot
                            && !member.roles.is_empty()
                        {
                            let result = member.remove_roles(&ctx.http, &member.roles).await;
                            removed_roles += 1;

                            if result.is_err() {
                                return format!(
                                    "Failed to remove roles from {}",
                                    member.display_name()
                                )
                                .to_string();
                            }
                        }
                    }

                    return format!(
                        "Removed roles on {removed_roles} members of {total_members} in server."
                    )
                    .to_string();
                }
                Err(_) => return "Failed to get memberlist for server".to_string(),
            }
        }
    }

    return "An error occurred while processing roles".to_string();
}

pub fn register() -> CreateCommand {
    CreateCommand::new("sweep")
        .description(
            "Sweep the current server and remove roles from members without the mandatory role.",
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

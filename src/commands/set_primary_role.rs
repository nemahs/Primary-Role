use serenity::all::*;

use crate::data::AppData;

fn get_option(name: &str, vec: &Vec<CommandDataOption>) -> Option<CommandDataOption> {
    for option in vec {
        if option.name == name {
            return Some(option.clone());
        }
    }

    return None;
}

pub async fn run(ctx: &Context, options: &CommandData, data: &mut AppData) -> String {
    let new_id = get_option("role_id", &options.options).unwrap();
    let new_id = new_id.value.as_role_id().unwrap();
    let guild_id = options.guild_id.unwrap();

    // Validate role exists
    if !guild_id
        .roles(&ctx.http)
        .await
        .unwrap()
        .contains_key(&new_id)
    {
        return "Given role is not in this server".to_string();
    }
    // Update database
    data.update_server_primary_role(&guild_id, &new_id).unwrap();

    return format!("Updated primary role to {}", new_id.get()).to_string();
}

pub fn register() -> CreateCommand {
    let id_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "role_id",
        "Role to become the new primary role",
    )
    .required(true);

    CreateCommand::new("primaryrole")
    .description("Set the primary role for this server that all members must have in order to be given another role.")
    .default_member_permissions(Permissions::ADMINISTRATOR)
    .add_option(id_option)
}

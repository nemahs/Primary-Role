use log::{info, debug, error};
use serenity::all::*;


use crate::{commands::{commands::DiscordCommand}, data::AppData};

pub struct SweepCommand;

const DISCORD_BATCH_SIZE: u64 = 1000;

#[async_trait]
impl DiscordCommand for SweepCommand {
    /// Sweep through all members of a given server, purging roles from anyone without the configured primary role.
    ///
    /// @param ctx Context object for the command being processed
    /// @param command Command being processed
    /// @param app_data Database of primary roles
    ///
    /// @return Result message to display to the user
    async fn run(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        app_data: &mut AppData,
    ) -> String {
        let Some(guild_id) = command.guild_id else {
            return "No server ID was given".to_string();
        };

        let Some(member_count) = ctx.http.get_guild_with_counts(guild_id).await.map_or(None, |guild| guild.approximate_member_count) else {
            return "Failed to get the member count for this server".to_string();
        };

        info!("Member count for server {} is {}", guild_id.get(), member_count);

        let Some(primary_role) = app_data.get_primary_role(&guild_id) else {
            return "Failed to determine the primary role for this server".to_string();
        };

        let Ok(member_list) = (match member_count {
            1..DISCORD_BATCH_SIZE => guild_id.members(&ctx, None, None).await,
            DISCORD_BATCH_SIZE.. => {
                let mut members: Vec<Member> = Vec::new();
                let mut last_id = None;

                loop {
                    let batch = guild_id
                        .members(&ctx, Some(DISCORD_BATCH_SIZE), last_id)
                        .await
                        .unwrap_or_default();
                    let batch_size = batch.len();


                    if batch_size == 0 {
                        break;
                    }

                    last_id = batch.last().map_or(None, |member| Some(member.user.id));

                    info!("Offset is now {:?}", last_id);
                    members.extend(batch);

                    if batch_size < 1000 {
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(50)).await; // Avoid hitting rate limits
                }

                Ok(members)
            },
            0 => {
                return "No members found in this server".to_string();
            }
        }) else {
            return "Failed to retrieve the list of members from the server".to_string();
        };

        let new_ctx = ctx.clone();
        let interaction = command.clone();
        let member_count = member_list.len();

        info!(
            "Starting a sweep of {} members in server {}",
            member_count,
            guild_id.get()
        );


        let sweep_function = async move {
            let mut removed_roles: u64 = 0;
            for member in member_list {
                debug!("Processing member {}", member.user.id);
                if !member.roles.contains(&primary_role)
                {
                    if member.user.bot {
                        debug!("Skipping bot user {}", member.user.id);
                        continue;
                    }

                    if member.roles.is_empty() {
                        debug!("Skipping user {} with no roles", member.user.id);
                        continue;
                    }

                    let result = member.remove_roles(&new_ctx, &member.roles).await;
                    removed_roles += 1;

                    if result.is_err() {
                        error!("Failed to remove roles from member {}", member.user.id);
                        continue;
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(25)).await; // Avoid hitting rate limits
            }


            let created_message = CreateMessage::new()
                    .content(format!(
                        "Completed sweeping through {} members, removed roles from {} members",
                        member_count, removed_roles
                    ));

            interaction.user.dm(&new_ctx, created_message).await.ok();
            info!(
                "Swept through {} members, removed roles from {} members",
                member_count, removed_roles
            );
        };

        tokio::spawn(sweep_function);

        return format!("Sweeping through {} members", member_count).to_string();
    }

    /// Create the command to register with Discord
    fn register(&self) -> CreateCommand {
        CreateCommand::new("sweep")
        .description(
            "Sweep the current server and remove roles from members without the mandatory role.",
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
    }
}

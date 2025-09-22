use log::{debug, error, info};
use serenity::all::*;

use crate::{commands::commands::DiscordCommand, data::AppData};

pub struct SweepCommand;

const DISCORD_BATCH_SIZE: u64 = 1000;

impl SweepCommand {
    async fn sweep(ctx: Context, command: CommandInteraction, members: Vec<Member>, primary_role: RoleId) {
        let member_count = members.len();
        let mut removed_roles: u64 = 0;
        for member in members {
            debug!("Processing member {}", member.user.id);

            if member.roles.contains(&primary_role) {
                continue; // User has the primary role, skip them
            }

            if member.user.bot {
                debug!("Skipping bot user {}", member.user.id);
                continue; // Do not touch any bots, bots aren't auto granted the primary role
            }

            if member.roles.is_empty() {
                debug!("Skipping user {} with no roles", member.user.id);
                continue;
            }

            match member.remove_roles(&ctx, &member.roles).await {
                Ok(_) => {
                    info!("Removed roles from {}", member.user.id);
                    removed_roles += 1;
                }
                Err(error) => {
                    error!("Failed to remove roles from {}: {}", member.user.id, error);
                    continue;
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(25)).await; // Avoid hitting rate limits
        }

        let created_message = CreateMessage::new().content(format!("Completed sweeping through {} members, removed roles from {} members", member_count, removed_roles));

        command.user.dm(&ctx, created_message).await.ok();
        info!("Swept through {} members, removed roles from {} members", member_count, removed_roles);
    }
}

#[async_trait]
impl DiscordCommand for SweepCommand {
    /// Sweep through all members of a given server, purging roles from anyone without the configured primary role.
    ///
    /// @param ctx Context object for the command being processed
    /// @param command Command being processed
    /// @param app_data Database of primary roles
    ///
    /// @return Result message to display to the user
    async fn run(&self, ctx: &Context, command: &CommandInteraction, app_data: &mut AppData) -> String {
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
                let mut last_id: Option<UserId> = None;

                loop {
                    let batch = guild_id.members(&ctx, Some(DISCORD_BATCH_SIZE), last_id).await.unwrap_or_default();
                    let batch_size = batch.len();

                    if batch_size == 0 {
                        break;
                    }

                    last_id = batch.last().map_or(None, |member| Some(member.user.id));

                    debug!("Offset is now {:?}", last_id);
                    members.extend(batch);

                    if batch_size < DISCORD_BATCH_SIZE as usize {
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    // Avoid hitting rate limits
                }

                Ok(members)
            }
            0 => {
                return "No members found in this server".to_string();
            }
        }) else {
            return "Failed to retrieve the list of members from the server".to_string();
        };

        let member_count = member_list.len();

        info!("Starting a sweep of {} members in server {}", member_count, guild_id.get());

        tokio::spawn(SweepCommand::sweep(ctx.clone(), command.clone(), member_list, primary_role));

        return format!("Sweeping through {} members", member_count).to_string();
    }

    /// Create the command to register with Discord
    fn register(&self) -> CreateCommand {
        CreateCommand::new("sweep")
            .description("Sweep the current server and remove roles from members without the mandatory role.")
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .add_context(InteractionContext::Guild)
    }
}

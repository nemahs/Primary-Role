use data::AppData;
use log::*;
use phf::phf_map;
use serenity::{all::*, async_trait, Client};
use std::fs;
use tokio::sync::Mutex;

use crate::commands::commands::DiscordCommand;

mod commands;
mod data;

struct Handler {
    app_data: Mutex<AppData>,
}

const COMMANDS: phf::Map<&'static str, &dyn DiscordCommand> = phf_map! {
    "enable" => &commands::bot_management::EnableCommand,
    "disable" => &commands::bot_management::DisableCommand,
    "status" => &commands::bot_management::StatusCommand,
    "sweep" => &commands::sweep::SweepCommand,
    "setprimaryrole" => &commands::primary_role::SetPrimaryRoleCommand,
    "getprimaryrole" => &commands::primary_role::GetPrimaryRoleCommand,
};

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        let mut app_data = self.app_data.lock().await;

        let command_func = COMMANDS.get(&command.data.name);
        let content = match command_func {
            Some(cmd) => Some(cmd.run(&ctx, &command, &mut app_data).await),
            None => {
                error!("No command found matching {}", command.data.name);
                None
            }
        };

        if let Some(content) = content {
            let data = CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true);
            let builder = CreateInteractionResponse::Message(data);

            if command.create_response(&ctx, builder).await.is_err() {
                error!("Failed to send response for command {}", command.data.name);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected to server successfully");

        for guild in ready.guilds {
            let guild = guild.id;
            let register_functions = COMMANDS
                .entries()
                .map(|(_, cmd)| cmd.register())
                .collect::<Vec<_>>();

            let registered_commands = guild.set_commands(&ctx, register_functions).await.ok();

            match registered_commands {
                Some(commands) => info!(
                    "Registered the following commands to {}: {commands:?}",
                    guild.get()
                ),
                None => error!("Failed to register commands to {}", guild.get()),
            };

            if self.app_data.lock().await.new_server(&guild).is_err() {
                error!("Could not add guild {} to the database", guild.get());
            }
        }
    }

    async fn guild_member_update(
        &self,
        ctx: Context,
        _old: Option<Member>,
        _new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        debug!("Got a guild member update");
        let app_data = self.app_data.lock().await;
        let is_auto_scan_enabled = app_data.is_auto_scan_enabled(&event.guild_id);
        if !is_auto_scan_enabled {
            return; // Do nothing, auto scan is disabled.
        }

        let primary_role = app_data.get_primary_role(&event.guild_id);
        let Some(primary_role) = primary_role else {
            error!("Failed to get primary role for {}", event.guild_id);
            return;
        };

        if !event.roles.contains(&primary_role) {
            // Remove all other roles
            let member = ctx
                .http
                .get_member(event.guild_id, event.user.id)
                .await
                .ok();

            let Some(member) = member else {
                error!("Could not find member for ID: {}", event.user.id);
                return;
            };

            match member.remove_roles(&ctx, &event.roles).await {
                Ok(_) => info!("Removed roles from {member:?}"),
                Err(_) => error!("Failed to remove roles from {member:?}"),
            };
        }
    }
}

const TOKEN_FILE: &str = "/run/secrets/DISCORD_TOKEN";
const DATABASE_FILE: &str = "/app/data/config.sqlite";

#[tokio::main]
async fn main() {
    env_logger::init();
    let token = fs::read_to_string(TOKEN_FILE).expect("Expected token file to exist");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            app_data: Mutex::new(AppData::new(DATABASE_FILE)),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

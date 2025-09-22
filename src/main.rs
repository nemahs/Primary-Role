use data::AppData;
use futures::future::OptionFuture;
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
    "sweep" => &commands::sweep::SweepCommand,
    "primaryrole" => &commands::primary_role::PrimaryRoleCommands,
    "scanning" => &commands::bot_management::ScanningCommands,
};

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        let mut app_data = self.app_data.lock().await;

        let command_func = COMMANDS.get(&command.data.name);
        let content = Into::<OptionFuture<_>>::into(command_func.map(|cmd| cmd.run(&ctx, &command, &mut app_data))).await;

        if let Some(content) = content {
            let data = CreateInteractionResponseMessage::new().content(content).ephemeral(true);
            let builder = CreateInteractionResponse::Message(data);

            command.create_response(&ctx, builder).await.unwrap_or_else(|error| {
                error!("Failed to send response for command {}: {}", command.data.name, error);
            });
        } else {
            error!("No command function found for {}", command.data.name);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected to server successfully");

        for guild in ready.guilds {
            let guild = guild.id;
            let register_functions = COMMANDS.entries().map(|(_, cmd)| cmd.register()).collect::<Vec<_>>();

            match guild.set_commands(&ctx, register_functions).await {
                Ok(registered_commands) => info!("Registered {} commands to {}", registered_commands.len(), guild.get()),
                Err(error) => error!("Failed to register commands to {}: {}", guild.get(), error),
            };

            self.app_data.lock().await.new_server(&guild).unwrap_or_else(|error| {
                error!("Could not add guild {} to the database: {}", guild.get(), error);
            });
        }
    }

    async fn guild_member_update(&self, ctx: Context, _old: Option<Member>, _new: Option<Member>, event: GuildMemberUpdateEvent) {
        debug!("Got a guild member update");
        let app_data = self.app_data.lock().await;

        if !app_data.is_auto_scan_enabled(&event.guild_id) {
            return; // Do nothing, auto scan is disabled.
        }

        let Some(primary_role) = app_data.get_primary_role(&event.guild_id) else {
            error!("Failed to get primary role for {}", event.guild_id);
            return;
        };

        if !event.roles.contains(&primary_role) {
            // Remove all other roles
            let Ok(member) = ctx.http.get_member(event.guild_id, event.user.id).await else {
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

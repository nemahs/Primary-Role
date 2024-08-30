use data::AppData;
use dotenv::dotenv;
use log::*;
use std::env;

mod data;

use serenity::{all::*, async_trait, Client};
use tokio::sync::Mutex;

mod commands;

struct Handler {
    app_data: Mutex<AppData>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let mut app_data = self.app_data.lock().await;

            let content = match command.data.name.as_str() {
                "enable" => Some(commands::enable::run(&app_data, &command)),
                "disable" => Some(commands::disable::run(&app_data, &command)),
                "sweep" => Some(commands::sweep::run(&ctx, &command, &self.app_data).await),
                "primaryrole" => {
                    Some(commands::set_primary_role::run(&ctx, &command.data, &mut app_data).await)
                }
                _ => None, // Not a valid command, leave it for another bot to deal with.
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true);
                let builder = CreateInteractionResponse::Message(data);

                command.create_response(&ctx.http, builder).await.unwrap();
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected to server successfully");

        for guild in ready.guilds {
            guild
                .id
                .set_commands(
                    &ctx.http,
                    vec![
                        commands::enable::register(),
                        commands::disable::register(),
                        commands::sweep::register(),
                        commands::set_primary_role::register(),
                    ],
                )
                .await
                .unwrap();

            self.app_data.lock().await.new_server(&guild.id).unwrap();
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
        let enabled = app_data.is_auto_scan_enabled(&event.guild_id);
        if !enabled {
            return; // Do nothing we aren't turned on.
        }

        let primary_role = app_data.get_primary_role(&event.guild_id);
        if let Some(role) = primary_role {
            if !event.roles.contains(&role) {
                // Remove all other roles
                let member = ctx
                    .http
                    .get_member(event.guild_id, event.user.id)
                    .await
                    .unwrap();

                member.remove_roles(ctx.http, &event.roles).await.unwrap();

                info!("Removed roles from {member:?}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN should be in either the .env or in an environment variable");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            app_data: Mutex::new(AppData::new()),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

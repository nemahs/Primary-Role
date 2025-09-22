use serenity::all::*;

use crate::data::AppData;

#[async_trait]
pub trait DiscordCommand: Send + Sync {
    fn register(&self) -> CreateCommand;

    async fn run(&self, ctx: &Context, command: &CommandInteraction, data: &mut AppData) -> String;
}

/// Retrieve a given option from the list of provided options
///
/// @param name Option name to find
/// @param vec List of provided command options
///
/// @return Option matching the given name, or None if not found
pub fn get_option(name: &str, vec: &Vec<CommandDataOption>) -> Option<CommandDataOption> {
    for option in vec {
        if option.name == name {
            return Some(option.clone());
        }
    }

    return None;
}

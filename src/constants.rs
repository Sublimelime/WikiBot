/// A file that holds all the constants that my bot uses.

extern crate serenity;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use self::serenity::model::GuildId;
use self::serenity::model::Message;

lazy_static! {
    pub static ref PREFIXES: Arc<Mutex<HashMap<GuildId, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub static BOT_NAME: &'static str = "WikiBot#3868";

/// Gets the current set prefix, given a message from a guild. {{{1
pub fn get_prefix_for_guild(message: &Message) -> String {
    let prefixes = PREFIXES.lock().unwrap();
    if let Some(pref) = prefixes.get(&message.guild_id().unwrap()) {
        pref.clone()
    } else {
        String::new()
    }
}

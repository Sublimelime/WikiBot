/// A file that holds all the constants that my bot uses.

extern crate serenity;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use self::serenity::model::GuildId;

lazy_static! {
    pub static ref PREFIXES: Arc<Mutex<HashMap<GuildId, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub static BOT_NAME: &'static str = "WikiBot#3868";

/// Gets the current set prefix, given a message from a guild. {{{1
pub fn get_prefix_for_guild(id: &GuildId) -> String {
    let prefixes = PREFIXES.lock().unwrap();
    if let Some(pref) = prefixes.get(&id) {
        pref.clone()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use self::serenity::model::GuildId;
    use super::*;

    #[test]
    fn guild_id_has_no_prefix() {
        // Dummy ID to try to retrieve from PREFIXES
        let id = GuildId::from(000000000000000000);
        let prefix = get_prefix_for_guild(&id);
        // Fail test if it returns a valid prefix
        assert_eq!(prefix, "");
    }

    #[test]
    fn adding_a_prefix_to_a_guild_id() {
        //Setup vars
        let mut prefixes = PREFIXES.lock().unwrap();
        let id = GuildId::from(111111111111111111);
        let _ = prefixes.insert(id, String::from("+"));

        let prefix = get_prefix_for_guild(&id);

        assert_eq!(prefix, "+");
    }
}

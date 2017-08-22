extern crate serenity;
extern crate reqwest;

use constants::*;
use common_funcs::*;
use levenshtein::*;
use self::serenity::model::Message;

/// Struct used to hold a bunch of data about a mod, for easy passing {{{1
#[derive(Debug)]
struct Mod {
    pub creation_date: String,
    pub name: String,
    pub author: String,
    pub link: String,
    pub download_count: u64,
    pub source_path: Option<String>,
    pub homepage: Option<String>,
    pub summary: String,
    pub tag: String,
    pub last_updated: String,
}

/// Creates an embed based on the recieved mod data. {{{1
fn make_mod_embed(modification: Mod, message: &Message) -> Result<(), String> {
    Ok(())
}

/// Links a mod into chat based on args. {{{1
command!(linkmod(_context, message) {
    let request = fix_message(message.content_safe(), "linkmod", &get_prefix_for_guild(&message.guild_id().unwrap()));


});

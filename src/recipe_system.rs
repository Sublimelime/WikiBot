extern crate serenity;
extern crate json;

use self::json::*;
use std::fs::File;
use commands::*;
use constants::*;
use levenshtein::*;

/// Returns a recipe from the game, either an item, machine, or tech. {{{1
/// Should be called with the name of the thing to get
command!(recipe(_context, message) {
    let request = fix_message(message.content_safe(), "recipe ", &get_prefix_for_guild(&message.guild_id().unwrap()));

});

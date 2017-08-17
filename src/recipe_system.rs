extern crate serenity;
extern crate json;

use self::json::*;
use std::fs::File;
use std::collections::BTreeMap;
use std::io::Read;
use commands::*;
use constants::*;
use levenshtein::*;

lazy_static! {
    static ref RECIPES: JsonValue = {
        if let Ok(mut file_result) = File::open("recipes.json") {
            let mut dest = String::new();
            if let Err(_) = file_result.read_to_string(&mut dest) {
                return JsonValue::new_object();
            }
            if let Ok(value) = json::parse(dest.as_str()) {
                return value;
            } else {
                return JsonValue::new_object();
            }
        } else {
            return JsonValue::new_object();
        }
    };
}

/// Returns a recipe from the game, either an item, machine, or tech. {{{1
/// Should be called with the name of the thing to get
command!(recipe(_context, message) {
    let request = fix_message(message.content_safe(), "recipe ", &get_prefix_for_guild(&message.guild_id().unwrap()));

});

// Functions {{{1
/// Gets the closest key match in RECIPES using levenshtein distance. {{{2
fn get_closest_match(request: &str) -> (usize, JsonValue) {

    // Quick exit
    if RECIPES.has_key(request) {
        return (0, RECIPES[request].clone());
    }
    let mut possiblities = BTreeMap::new();

    // Add keys to the treemap corresponding to levenshtein distance
    for entry in RECIPES.entries() {
        let (key, value) = entry;
        possiblities.insert(levenshtein(request, key), value);
    }
    //return the smallest key, unwrap because there should be a value
    let (dist, smallest) = possiblities.iter_mut().next().unwrap();
    return (*dist, smallest.clone());

}

// Tests {{{1
// TODO Write tests for this file
#[cfg(test)]
mod tests {
    #[test]
    fn closest_match_with_exact() {}

    #[test]
    fn closest_match_with_slightly_incorrect() {}
}

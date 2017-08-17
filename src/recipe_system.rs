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
    // Fail out if RECIPES failed to init
    if RECIPES.is_empty() {
        say_into_chat(&message, "Sorry, I couldn't get a list of recipes in the game correctly. This command isn't going to work.");
    } else {
        let _ = message.channel_id.broadcast_typing();
        let request = fix_message(message.content_safe(), "recipe ", &get_prefix_for_guild(&message.guild_id().unwrap()));
        let (dist, closest_match) = get_closest_match(&request);

        // If it wasn't an exact match
        if dist != 0 {


        } else {
            let result = message.channel_id.send_message(|a| a
                                                         .embed(|b| b
                                                                .title(&format!("Recipe for {}", closest_match["wiki-name"]))
                                                                .field(|c| c
                                                                       .name("Required crafting energy")
                                                                       .value(&format!("{}", closest_match["energy-required"])))
                                                                .field(|c| c
                                                                       .name("Inputs")
                                                                       .value(&serialize_recipe_io(&closest_match["inputs"])))
                                                                .field(|c| c
                                                                       .name("Outputs")
                                                                       .value(&serialize_recipe_io(&closest_match["outputs"])))
                                                               )
                                                        );
            if let Err(error) = result {
                say_into_chat(&message, "Sorry, I couldn't make an embed here. Contact an admin.");
            }
        }
    }
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

/// Serializes the inputs or outputs object it recieves into a nice list. {{{2
fn serialize_recipe_io(value: &JsonValue) -> String {
String::new()

}

// Tests {{{1
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_match_with_exact() {
        let request = "beacon";
        let (dist, closest_match) = get_closest_match(request);
        assert_eq!(closest_match["wiki-name"], RECIPES["beacon"]["wiki-name"]);
        assert_eq!(dist, 0);
    }

    #[test]
    fn closest_match_with_slightly_incorrect() {
        let request = "baecon";
        let (dist, closest_match) = get_closest_match(request);
        assert_eq!(closest_match["wiki-name"], RECIPES["beacon"]["wiki-name"]);
        assert_eq!(dist, 2);
    }

    #[test]
    fn can_serialize_recipe_io() {
        let mut testing_object = JsonValue::new_object();
        // Add some values
        testing_object["copper-plate"] = 2;
        testing_object["iron-gear-wheel"] = 6;
        testing_object["raw-wood"] = 10;

        let result = serialize_recipe_io(&testing_object);
        assert_eq!(result, "Copper plate x2, Iron gear wheel x6, Raw wood x10");
    }
}

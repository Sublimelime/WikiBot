use json;
use json::JsonValue;

use serenity::utils::Colour;

use std::fs::File;
use std::io::Read;

use common_funcs::*;
use constants::*;

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
        let request = fix_message(message.content_safe(), "recipe");

        //Bail out if there's no argument
        if request.is_empty() {
            if let Err(_) = send_error_embed(&message, "You must provide the name of an item, process or entity here, it will be autocorrected if it's slightly off.") {
                say_into_chat(&message, "You must provide the name of an item, process or entity here, it will be autocorrected if it's slightly off.");
            }
            return Err(String::from("Missing argument, failed."));
        }
        // Create a list of just keys
        let recipe_key_list: Vec<&str> = RECIPES.entries().map(|(key, _value)| {
            key
        }).collect();

        // Find the closest match to what they asked for
        let (dist, closest_match) = get_closest_match(&recipe_key_list, &request);

        // Bail out if the distance is too great
        if dist >= DISTANCE_SENSITIVITY {
            if let Err(_) = send_error_embed(&message, "Sorry, I couldn't find any recipe for that request. Does the object you're asking for go by any other name?") {
                say_into_chat(&message, "Sorry, I couldn't find any recipe for that request. Does the object you're asking for go by any other name?");
            }

            return Err(String::from("Missing recipe, failed."));
        }

        // If it wasn't an exact match
        if dist != 0 {
            let result = message.channel_id.send_message(|a| a
                                                         .embed(|b| b
                                                                .title(&format!("Recipe for {}: (closest, distance {})", RECIPES[closest_match]["wiki-name"], dist))
                                                                .field(|c| c
                                                                       .name("Crafting time")
                                                                       .value(&format!("{}", RECIPES[closest_match]["energy-required"])))
                                                                .field(|c| c
                                                                       .name("Inputs")
                                                                       .value(&serialize_recipe_io(&RECIPES[closest_match]["inputs"])))
                                                                .field(|c| c
                                                                       .name("Outputs")
                                                                       .value(&serialize_recipe_io(&RECIPES[closest_match]["outputs"])))
                                                                .timestamp(message.timestamp.to_rfc3339())
                                                                .color(Colour::from_rgb(10, 225, 249))
                                                               ));
            if let Err(_) = result {
                say_into_chat(&message, "Sorry, I couldn't make an embed here. Contact an admin.");
            }

        } else {
            let result = message.channel_id.send_message(|a| a
                                                         .embed(|b| b
                                                                .title(&format!("Recipe for {}:", RECIPES[closest_match]["wiki-name"]))
                                                                .field(|c| c
                                                                       .name("Crafting time")
                                                                       .value(&format!("{}", RECIPES[closest_match]["energy-required"])))
                                                                .field(|c| c
                                                                       .name("Inputs")
                                                                       .value(&serialize_recipe_io(&RECIPES[closest_match]["inputs"])))
                                                                .field(|c| c
                                                                       .name("Outputs")
                                                                       .value(&serialize_recipe_io(&RECIPES[closest_match]["outputs"])))
                                                                .timestamp(message.timestamp.to_rfc3339())
                                                                .color(Colour::from_rgb(10, 225, 249))
                                                               ));
            if let Err(_) = result {
                say_into_chat(&message, "Sorry, I couldn't make an embed here. Contact an admin.");
            }
        }
    }
});

// Functions {{{1
/// Serializes the inputs or outputs object it recieves into a nice list. {{{2
fn serialize_recipe_io(value: &JsonValue) -> String {
    if value.is_empty() {
        return String::from("None");
    }
    let mut result = String::new();
    for entry in value.entries() {
        let (ing_key, ingredient) = entry;
        let mut pretty_name = format!("{}", RECIPES[ing_key]["wiki-name"]);
        // Check if there is a pretty name for the ingredient, otherwise use internal
        if pretty_name == "null" {
            pretty_name = String::from(ing_key);
        }
        result += &format!("{} x{}, ", pretty_name, ingredient);
    }
    if result.len() > 3 {
        return result[..result.len() - 2].to_owned();
    }
    result
}

// Tests {{{1
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_serialize_recipe_io() {
        let mut testing_object = JsonValue::new_object();
        // Add some values
        testing_object["copper-plate"] = 2.into();
        testing_object["iron-gear-wheel"] = 6.into();
        testing_object["raw-wood"] = 10.into();

        let result = serialize_recipe_io(&testing_object);
        assert_eq!(result, "Copper plate x2, Iron gear wheel x6, raw-wood x10");
    }
}

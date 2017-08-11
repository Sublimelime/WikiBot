extern crate serenity;
extern crate rand;
extern crate json;

use self::serenity::model::Message;
use self::json::JsonValue;
use self::serenity::utils::Colour;
use std::fs::File;
use std::io::prelude::*;

use commands::*;
use levenshtein::*;

/// Prints out a grand list of all current stored ratios.
command!(ratios(_context, message) {
    let parsed_json = get_ratio_json();

    let mut embed_content = String::new();

    for entry in parsed_json.entries() {
        //Destructure the tuple to make code easier
        let (key, value) = entry;

        //Change from proprietary format into native
        embed_content += format!("{}\n{}\n\n", key, value.as_str().unwrap()).as_str();
    }

    //Send the message with embed
    let _ = message.channel_id.send_message(|a| a
                                            .content("List of all registered ratios:")
                                            .embed(|b| b
                                                   .field(|c| c.name("List:").value(&embed_content[..]))
                                                   .author(|d| d
                                                           .name("WikiBot")
                                                           .url("https://bitbucket.com/Gangsir")
                                                          )
                                                   .color(Colour::from_rgb(100,200,100))
                                                  ));
});

/// Adds a ratio to the list of current ratios.
/// Can only be used by moderators.
command!(ratio_add(_context, message, _args, name: String, ratio: String) {
    // Reject if they don't use quotes, since the ratio wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() < 4 {
        say_into_chat(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help ratio add``` for info on how to format this command.", ::PREFIX).as_str());
    } else {
        let mut parsed_json = get_ratio_json();

        if !parsed_json.has_key(name.as_str()) {

            // Add the entry to the json
            parsed_json[&name] = ratio.clone().into();

            // Write it back to the file
            write_ratio_json(parsed_json);

            say_into_chat(&message, format!("Success, added ratio {} for concept {}.", ratio, name).as_str());
        } else {
            say_into_chat(&message, "Cannot add, dictionary already contains an entry for that name. Try using ```ratio set``` instead, or removing it.");
        }
    }
});

/// Retrieves a ratio from the storage of the bot.
command!(ratio_get(_context, message) {
    let parsed_json = get_ratio_json();
    let request = fix_message(message.content_safe(), "ratio get ");

    // Key is not found, do a levenshtein search to see if they made a typo
    if !parsed_json.has_key(request.as_str()) {
        let mut possiblities: Vec<&str> = Vec::new();

        for entry in parsed_json.entries() {
            let (key, _value) = entry;
            if levenshtein(key, request.as_str()) < 5 {
                possiblities.push(key);
            }
        }
        say_into_chat(&message, format!("Sorry, I didn't find anything for `{}`. Did you mean one of the following?\n{:#?}", request, possiblities).as_str());
    } else { // Key is found literally
        // Build message
        say_into_chat(&message, format!("Ratio for {}:\n{}", request, parsed_json[&request].as_str().unwrap()).as_str());
    }
});

/// Deletes a stored ratio. Administrators only.
command!(ratio_delete(_context, message) {
    let mut parsed_json = get_ratio_json();
    let request = fix_message(message.content_safe(), "ratio delete ");

    if !parsed_json.has_key(request.as_str()) {
        say_into_chat(&message, format!("Sorry, I didn't find anything for `{}`. It might've been already deleted.", request).as_str());
    } else { // Key is found
        // Do the deletion
        let _ = parsed_json.remove(request.as_str());
        write_ratio_json(parsed_json);

        // Build message
        say_into_chat(&message, format!("Success, ratio for `{}` was deleted.", request).as_str());
    }
});

/// Deletes all stored ratios in the registry. Admin only.
command!(ratio_deleteall(_context, message) {
    // Clear all the ratios by writing an empty object to the file
    write_ratio_json(JsonValue::new_object());

    say_into_chat(&message, "Success, all ratios deleted.");
});


/// Changes the value of an existant ratio. Administrators only.
command!(ratio_set(_context, message, _args, name: String, ratio: String) {
    // Reject if they don't use quotes, since the ratio wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() < 4 {
        say_into_chat(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help ratio set``` for info on how to format this command.", ::PREFIX).as_str());
    } else {
        let mut parsed_json = get_ratio_json();

        if parsed_json.has_key(name.as_str()) {

            // Modify the entry
            parsed_json[&name] = ratio.clone().into();

            // Write it back to the file
            write_ratio_json(parsed_json);

            say_into_chat(&message, format!("Success, set ratio `{}` for concept `{}`.", ratio, name).as_str());
        } else {
            say_into_chat(&message, "Cannot set, key not found in dictionary. Try using ```ratio add``` instead.");
        }
    }
});

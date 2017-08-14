extern crate serenity;
extern crate rand;
extern crate json;

use self::json::JsonValue;
use self::serenity::utils::Colour;
use self::serenity::model::GuildId;

use commands::*;
use constants::*;
use std::fs::OpenOptions;
use levenshtein::*;

/// Prints out a grand list of all current stored ratios.
command!(ratios(_context, message) {
    let parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);

    let mut embed_content = String::new();

    for entry in parsed_json.entries() {
        //Destructure the tuple to make code easier
        let (key, _value) = entry;

        //Change from proprietary format into native
        embed_content += format!("{}, ", key).as_str();
    }

    //Send the message with embed
    let result = message.channel_id.send_message(|a| a
                                                 .content("List of all registered ratios:")
                                                 .embed(|b| b
                                                        .title("Ratios for this server:")
                                                        .description(embed_content.as_str())
                                                        .color(Colour::from_rgb(100,200,100))
                                                        .timestamp(message.timestamp.to_rfc3339())
                                                       ));
    if let Err(error) = result {
        println!("Got error sending list of ratios, error is: {:?}, parsed json is {}", error, parsed_json.dump());
        let _ = send_error_embed(&message, "Failed to get ratio list. Either something went wrong, or no ratios are defined.");
    }
});

/// Adds a ratio to the list of current ratios.
/// Can only be used by moderators.
command!(ratio_add(_context, message, _args, name: String, ratio: String) {
    // Reject if they don't use quotes, since the ratio wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() != 4 {
        let _ = send_error_embed(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help ratio add``` for info on how to format this command.", get_prefix_for_guild(&message)).as_str());
    } else {
        let mut parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);

        if !parsed_json.has_key(name.as_str()) {

            // Add the entry to the json
            parsed_json[&name] = ratio.clone().into();

            // Write it back to the file
            write_ratio_json(parsed_json, &message.guild_id().unwrap());

            if let Err(_) = send_success_embed(&message, format!("Success, added ratio {} for concept {}.", ratio, name).as_str()) {
                say_into_chat(&message, format!("Success, added ratio {} for concept {}.", ratio, name));
            }
        } else {
            let _ = send_error_embed(&message, "Cannot add, dictionary already contains an entry for that name. Try using ```ratio set``` instead, or removing it.");
        }
    }
});

/// Retrieves a ratio from the storage of the bot.
command!(ratio_get(_context, message) {
    let parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);
    let request = fix_message(message.content_safe(), "ratio get ", &message);

    // Key is not found, do a levenshtein search to see if they made a typo
    if !parsed_json.has_key(request.as_str()) {
        let mut possiblities: Vec<&str> = Vec::new();

        for entry in parsed_json.entries() {
            let (key, _value) = entry;
            if levenshtein(key, request.as_str()) < 5 {
                possiblities.push(key);
            }
        }
        // TODO make the output of this better
        send_error_embed(&message, format!("Sorry, I didn't find anything for `{}`. Did you mean one of the following?\n{:#?}", request, possiblities).as_str());
    } else { // Key is found literally
        // Build message
        say_into_chat(&message, format!("Ratio for {}:\n{}", request, parsed_json[&request].as_str().unwrap()));
    }
});

/// Deletes a stored ratio. Administrators only.
command!(ratio_delete(_context, message) {
    let mut parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);
    let request = fix_message(message.content_safe(), "ratio delete ", &message);

    if !parsed_json.has_key(request.as_str()) {
        let _ = send_error_embed(&message, format!("Sorry, I didn't find anything for `{}`. It might've been already deleted.", request).as_str());
    } else { // Key is found
        // Do the deletion
        let _ = parsed_json.remove(request.as_str());
        write_ratio_json(parsed_json, &message.guild_id().unwrap());

        // Build message
        if let Err(_) = send_success_embed(&message, format!("Success, ratio for `{}` was deleted.", request).as_str()) {
            say_into_chat(&message, format!("Success, ratio for `{}` was deleted.", request));
        }
    }
});

/// Deletes all stored ratios in the registry. Admin only.
command!(ratio_deleteall(_context, message) {
    // Clear all the ratios by writing an empty object to the file
    write_ratio_json(JsonValue::new_object(), &message.guild_id().unwrap());

    if let Err(_) = send_success_embed(&message, "Success, all ratios deleted.") {
        say_into_chat(&message, "Success, all ratios deleted.");
    }
});


/// Changes the value of an existant ratio. Administrators only.
command!(ratio_set(_context, message, _args, name: String, ratio: String) {
    // Reject if they don't use quotes, since the ratio wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() != 4 {
        let _ = send_error_embed(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help ratio set``` for info on how to format this command.", get_prefix_for_guild(&message)).as_str());
    } else {
        let mut parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);

        if parsed_json.has_key(name.as_str()) {

            // Modify the entry
            parsed_json[&name] = ratio.clone().into();

            // Write it back to the file
            write_ratio_json(parsed_json, &message.guild_id().unwrap());

            if let Err(_) = send_success_embed(&message, format!("Success, set ratio `{}` for concept `{}`.", ratio, name).as_str()) {
                say_into_chat(&message, format!("Success, set ratio `{}` for concept `{}`.", ratio, name));
            }
        } else {
            let _ = send_error_embed(&message, "Cannot set, key not found in dictionary. Try using ```ratio add``` instead.");
        }
    }
});

///Takes a JsonValue, and writes it to the ratios file.
pub fn write_ratio_json(value: JsonValue, guild: &GuildId) {
    // Determine file name based on the guild in question
    let ratio_file = format!("{:?}-ratios.json", guild);

    // Open the json file for writing, nuking any previous contents
    let mut file = OpenOptions::new()
        .write(true)
        .create(true) //create the file if it doesn't exist
        .truncate(true)
        .open(ratio_file.as_str())
        .expect("Unable to open/create json file for writing.");

    // Write json to file
    if let Err(error) = value.write(&mut file) {
        make_log_entry(format!(
            "Error writing to json file,
            aborting with error: {:?}",
            error
        ), "Error");
    } else {
        make_log_entry(format!("Wrote to ratio file: {}", ratio_file), "Info");
    }
}

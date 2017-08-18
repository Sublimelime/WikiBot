extern crate serenity;
extern crate rand;
extern crate json;

use self::json::JsonValue;
use self::serenity::utils::Colour;
use self::serenity::model::GuildId;

use common_funcs::*;
use constants::*;
use std::fs::OpenOptions;
use levenshtein::*;

/// Prints out a grand list of all current stored ratios. {{{1
command!(ratios(_context, message) {
    let parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);

    let mut embed_content = String::new();

    embed_content = jsonvalue_as_comma_list(&parsed_json);

    //Send the message with embed
    let result = message.channel_id.send_message(|a| a
                                                 .content("List of all registered ratios:")
                                                 .embed(|b| b
                                                        .title("Ratios for this server:")
                                                        .description(embed_content.as_str())
                                                        .color(Colour::from_rgb(119,0,255))
                                                        .timestamp(message.timestamp.to_rfc3339())
                                                       ));
    if let Err(error) = result {
        make_log_entry(format!("Got error sending list of ratios, error is: {:?}, parsed json is {}", error, parsed_json.dump()), "Error");
        let _ = send_error_embed(&message, "Failed to get ratio list. Either something went wrong, or no ratios are defined.");
    }
});

/// Adds a ratio to the list of current ratios. {{{1
/// Can only be used by moderators.
command!(ratio_add(_context, message, _args, name: String, ratio: String) {
    // Reject if they don't use quotes, since the ratio wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() != 4 || name.is_empty() || ratio.is_empty() {
        let _ = send_error_embed(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help ratio-add``` for info on how to format this command.",
                                        get_prefix_for_guild(&message.guild_id().unwrap())
                                        ).as_str());
    } else {
        let mut parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);

        if !parsed_json.has_key(name.as_str()) {

            // Add the entry to the json
            parsed_json[&name] = ratio.clone().into();

            // Write it back to the file
            write_ratio_json(parsed_json, &message.guild_id().unwrap());

            if let Err(_) = send_success_embed(&message, format!("Success, added ratio `{}` for concept `{}`.", ratio, name).as_str()) {
                say_into_chat(&message, format!("Success, added ratio `{}` for concept `{}`.", ratio, name));
            }
        } else {
            let _ = send_error_embed(&message, "Cannot add, dictionary already contains an entry for that name. Try using ```ratio-set``` instead, or removing it.");
        }
    }
});

/// Retrieves a ratio from the storage of the bot. {{{1
command!(ratio_get(_context, message) {
    let parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);
    let request = fix_message(message.content_safe(), "ratio", &get_prefix_for_guild(&message.guild_id().unwrap()));

    if request.is_empty() {
        say_into_chat(&message, "Sorry, I was expecting the name of a ratio here. For a list of all ratios, use the `ratio-list` command.");
        return Ok(());
    }

    // Key is not found, do a levenshtein search to see if they made a typo
    if !parsed_json.has_key(request.as_str()) {
        let mut possiblities: Vec<&str> = Vec::new();

        for entry in parsed_json.entries() {
            let (key, _value) = entry;
            if levenshtein(key, request.as_str()) < 5 {
                possiblities.push(key);
            }
        }
        let possiblities_pretty = String::from(format!("{:?}", possiblities)).replace("[", "").replace("]", "");
        if let Err(_) = send_error_embed(&message, format!("Sorry, I didn't find anything for `{}`. Did you mean one of the following?\n{}",
                                                           request,
                                                           possiblities_pretty
                                                          ).as_str()) {
            say_into_chat(&message, format!("Unable to make embed, using fallback list: {:#?}", possiblities)) ;
        }
    } else { // Key is found literally
        // Build message
        if let Err(_) = message.channel_id.send_message(|a| a
                                                        .embed(|b| b
                                                               .title(request.as_str())
                                                               .description(parsed_json[&request].as_str().unwrap())
                                                               .color(Colour::from_rgb(119,0,255))
                                                               .timestamp(message.timestamp.to_rfc3339())
                                                              )) {
            say_into_chat(&message, format!("Ratio for `{}`:\n```{}```", request, parsed_json[&request].as_str().unwrap()));
        }
    }
});

/// Deletes a stored ratio. Administrators only. {{{1
command!(ratio_delete(_context, message) {
    let mut parsed_json = get_ratio_json(&message.guild_id().unwrap(), &message);
    let request = fix_message(message.content_safe(), "ratio-delete ", &get_prefix_for_guild(&message.guild_id().unwrap()));

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

/// Deletes all stored ratios in the registry. Admin only. {{{1
command!(ratio_deleteall(_context, message) {
    // Clear all the ratios by writing an empty object to the file
    write_ratio_json(JsonValue::new_object(), &message.guild_id().unwrap());

    if let Err(_) = send_success_embed(&message, "Success, all ratios deleted.") {
        say_into_chat(&message, "Success, all ratios deleted.");
    }
});


/// Changes the value of an existant ratio. Administrators only. {{{1
command!(ratio_set(_context, message, _args, name: String, ratio: String) {
    // Reject if they don't use quotes, since the ratio wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() != 4 || name.is_empty() || ratio.is_empty() {
        let _ = send_error_embed(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help ratio-set``` for info on how to format this command.",
                                        get_prefix_for_guild(&message.guild_id().unwrap())
                                        ).as_str());
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

///Takes a JsonValue, and writes it to the ratios file. {{{1
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
        make_log_entry(
            format!(
                "Error writing to json file,
            aborting with error: {:?}",
                error
            ),
            "Error",
        );
    } else {
        make_log_entry(format!("Wrote to ratio file: {}", ratio_file), "Info");
    }
}

/// Takes a JsonValue, and returns a string of all the keys {{{1
/// in a comma seperated list.
fn jsonvalue_as_comma_list(json_value: &JsonValue) -> String {
    let mut result = String::new();
    for entry in json_value.entries() {
        //Destructure the tuple to make code easier
        let (key, _value) = entry;

        //Change from proprietary format into native
        result += format!("{}, ", key).as_str();
    }

    // Remove comma from end if able
    if result.len() > 3 {
        result = String::from(&result[..result.len() - 2]);
    }
    result
}

// Tests {{{1
#[cfg(test)]
mod tests {
    extern crate json;
    use self::json::JsonValue;
    use super::*;

    // Tests if code can read a list of keys and make a nice list {{{2
    #[test]
    fn can_parse_json_into_list() {
        let mut testing_object = JsonValue::new_object();
        // push some values to the object
        testing_object["first"] = "value".into();
        testing_object["second"] = "value".into();
        testing_object["third"] = "value".into();
        testing_object["fourth"] = "value".into();

        let listed = jsonvalue_as_comma_list(&testing_object);
        // Test if the result is what we want
        assert_eq!(listed, "first, second, third, fourth");
    }
}

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

/// Prints out a grand list of all current stored faqs. {{{1
command!(faqs(_context, message) {
    let parsed_json = get_faq_json(&message.guild_id().unwrap(), &message);

    let mut embed_content;

    embed_content = jsonvalue_as_comma_list(&parsed_json);

    //Send the message with embed
    let result = message.channel_id.send_message(|a| a
                                                 .content("List of all registered faqs:")
                                                 .embed(|b| b
                                                        .title("faqs for this server:")
                                                        .description(embed_content.as_str())
                                                        .color(Colour::from_rgb(119,0,255))
                                                        .timestamp(message.timestamp.to_rfc3339())
                                                       ));
    if let Err(error) = result {
        let _ = send_error_embed(&message, "Failed to get faq list. Either something went wrong, or no faqs are defined.");
        return Err(format!("Got error sending list of faqs, error is: {:?}, parsed json is {}", error, parsed_json.dump()));
    }
});

/// Adds a faq to the list of current faqs. {{{1
/// Can only be used by moderators.
command!(faq_add(_context, message, _args, name: String, faq: String) {
    // Reject if they don't use quotes, since the faq wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() != 4 || name.is_empty() || faq.is_empty() {
        let _ = send_error_embed(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help faq-add``` for info on how to format this command.",
                                        get_prefix_for_guild(&message.guild_id().unwrap())
                                        ).as_str());
        return Err(String::from("Could not add due to missing quotes or invalid args."));
    }
    let mut parsed_json = get_faq_json(&message.guild_id().unwrap(), &message);
    let name = name.to_lowercase();

    if !parsed_json.has_key(name.as_str()) {

        // Add the entry to the json
        let file = message.attachments.get(0);
        if let Some(image) = file {
            let mut array = JsonValue::new_array();
            array.push(faq.clone());
            array.push(image.url.clone());
            parsed_json[&name] = array;
        } else {
            let mut array = JsonValue::new_array();
            array.push(faq.clone());
            parsed_json[&name] = array;
        }

        // Write it back to the file
        write_faq_json(parsed_json, &message.guild_id().unwrap());

        if let Err(_) = send_success_embed(&message, format!("Success, added faq `{}` for concept `{}`.", faq, name).as_str()) {
            say_into_chat(&message, format!("Success, added faq `{}` for concept `{}`.", faq, name));
        }
    } else {
        let _ = send_error_embed(&message, "Cannot add, dictionary already contains an entry for that name.
                                 Try using ```faq-set``` instead, or removing it.");
        return Err(String::from("Could not add due to an already existing key for provided faq."));
    }
});

/// Retrieves a faq from the storage of the bot. {{{1
command!(faq_get(_context, message, _args) {
    let parsed_json = get_faq_json(&message.guild_id().unwrap(), &message);
    let server_prefix = get_prefix_for_guild(&message.guild_id().unwrap());
    let mut request = String::new();

    if message.content_safe().starts_with(format!("{}faq", server_prefix).as_str()) {
        request = fix_message(message.content_safe(), "faq", &server_prefix);
    } else if message.content_safe().starts_with(format!("{}faw", server_prefix).as_str()) {
        request = fix_message(message.content_safe(), "faw", &server_prefix);
    }

    if request.is_empty() {
        // Call the other command's function, since the user is looking for a list
        return faqs(_context, message, _args);
    }

    // Make lowercase
    request = request.to_lowercase();

    // Key is not found, do a levenshtein search to see if they made a typo
    if !parsed_json.has_key(request.as_str()) {
        let mut possiblities: Vec<&str> = Vec::new();

        for entry in parsed_json.entries() {
            let (key, _value) = entry;
            if levenshtein(key, request.as_str()) < 5 {
                possiblities.push(key);
            }
        }
        // Clean up the output
        let possiblities_pretty = String::from(format!("{:?}", possiblities)).replace("[", "").replace("]", "").replace('"', "");
        if let Err(_) = send_error_embed(&message, format!("Sorry, I didn't find anything for `{}`. Did you mean one of the following?\n{}",
                                                           request,
                                                           possiblities_pretty
                                                          ).as_str()) {
            say_into_chat(&message, format!("Unable to make embed, using fallback list: {:#?}", possiblities)) ;
        }
    } else { // Key is found literally
        if parsed_json[&request].len() > 1 {
            // Build message
            if let Err(_) = message.channel_id.send_message(|a| a
                                                            .embed(|b| b
                                                                   .title(request.as_str())
                                                                   .description(parsed_json[&request][0].as_str().unwrap())
                                                                   .image(parsed_json[&request][1].as_str().unwrap())
                                                                   .color(Colour::from_rgb(119,0,255))
                                                                   .timestamp(message.timestamp.to_rfc3339())
                                                                  )) {
                say_into_chat(&message, format!("FAQ for `{}`:\n```{}```\n(Normally there would be an image here.)", request, parsed_json[&request][0].as_str().unwrap()));
            }

        } else if parsed_json[&request].len() == 1 {
            // Build message
            if let Err(_) = message.channel_id.send_message(|a| a
                                                            .embed(|b| b
                                                                   .title(request.as_str())
                                                                   .description(parsed_json[&request][0].as_str().unwrap())
                                                                   .color(Colour::from_rgb(119,0,255))
                                                                   .timestamp(message.timestamp.to_rfc3339())
                                                                  )) {
                say_into_chat(&message, format!("FAQ for `{}`:\n```{}```", request, parsed_json[&request][0].as_str().unwrap()));
            }
        }
    }
});

/// Deletes a stored faq. Administrators only. {{{1
command!(faq_delete(_context, message) {
    let mut parsed_json = get_faq_json(&message.guild_id().unwrap(), &message);
    let request = fix_message(message.content_safe(), "faq-delete ", &get_prefix_for_guild(&message.guild_id().unwrap()));

    if !parsed_json.has_key(request.as_str()) {
        let _ = send_error_embed(&message, format!("Sorry, I didn't find anything for `{}`. It might've been already deleted.", request).as_str());
    } else { // Key is found
        // Do the deletion
        let _ = parsed_json.remove(request.as_str());
        write_faq_json(parsed_json, &message.guild_id().unwrap());

        // Build message
        if let Err(_) = send_success_embed(&message, format!("Success, faq for `{}` was deleted.", request).as_str()) {
            say_into_chat(&message, format!("Success, faq for `{}` was deleted.", request));
        }
    }
});

/// Deletes all stored faqs in the registry. Admin only. {{{1
command!(faq_deleteall(_context, message) {
    // Clear all the faqs by writing an empty object to the file
    write_faq_json(JsonValue::new_object(), &message.guild_id().unwrap());

    if let Err(_) = send_success_embed(&message, "Success, all faqs deleted.") {
        say_into_chat(&message, "Success, all faqs deleted.");
    }
});


/// Changes the value of an existant faq. Administrators only. {{{1
command!(faq_set(_context, message, _args, name: String, faq: String) {
    // Reject if they don't use quotes, since the faq wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() != 4 || name.is_empty() || faq.is_empty() {
        let _ = send_error_embed(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help faq-set``` for info on how to format this command.",
                                        get_prefix_for_guild(&message.guild_id().unwrap())
                                        ).as_str());
        return Err(String::from("Could not set faq due to invalid args."));
    } else {
        let mut parsed_json = get_faq_json(&message.guild_id().unwrap(), &message);
        let name = name.to_lowercase();

        if parsed_json.has_key(name.as_str()) {

            // Modify the entry
            let file = message.attachments.get(0);
            if let Some(image) = file {
                let mut array = JsonValue::new_array();
                let _ = array.push(faq.clone());
                let _ = array.push(image.url.clone());
                parsed_json[&name] = array;
            } else {
                let mut array = JsonValue::new_array();
                let _ = array.push(faq.clone());
                parsed_json[&name] = array;
            }

            // Write it back to the file
            write_faq_json(parsed_json, &message.guild_id().unwrap());

            if let Err(_) = send_success_embed(&message, format!("Success, set faq `{}` for concept `{}`.", faq, name).as_str()) {
                say_into_chat(&message, format!("Success, set faq `{}` for concept `{}`.", faq, name));
            }
        } else {
            let _ = send_error_embed(&message, "Cannot set, key not found in dictionary. Try using ```faq add``` instead.");
            return Err(String::from("Could not set faq due to missing key."));
        }
    }
});

///Takes a JsonValue, and writes it to the faqs file. {{{1
pub fn write_faq_json(value: JsonValue, guild: &GuildId) {
    // Determine file name based on the guild in question
    let faq_file = format!("{:?}-faqs.json", guild);

    // Open the json file for writing, nuking any previous contents
    let mut file = OpenOptions::new()
        .write(true)
        .create(true) //create the file if it doesn't exist
        .truncate(true)
        .open(faq_file.as_str())
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
        make_log_entry(format!("Wrote to json file: {}", faq_file), "Info");
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

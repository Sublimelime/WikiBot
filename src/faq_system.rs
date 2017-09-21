use json::{self, JsonValue};

use serenity::utils::Colour;
use serenity::model::{Message, GuildId};

use std::fs::{File, OpenOptions};
use std::io::prelude::*;

use common_funcs::*;
use constants::*;

/// Prints out a grand list of all current stored faqs. {{{1
command!(faqs(_context, message) {
    let _ = message.channel_id.broadcast_typing();
    let parsed_json = get_faq_json(&message.guild_id().unwrap(), &message);

    let mut embed_content;

    embed_content = jsonvalue_as_comma_list(&parsed_json);

    //Send the message with embed
    let result = message.channel_id.send_message(|a| a
                                                 .embed(|b| b
                                                        .title("FAQs for this server:")
                                                        .description(&embed_content)
                                                        .color(Colour::from_rgb(119,0,255))
                                                        .timestamp(message.timestamp.to_rfc3339())
                                                       ));
    if let Err(error) = result {
        send_error_embed_or_say(&message, "Failed to get faq list. Either something went wrong, or no faqs are defined.");
        return Err(format!("Got error sending list of faqs, error is: {:?}, parsed json is {}", error, parsed_json.dump()));
    }
});

/// Adds a faq to the list of current faqs. {{{1
/// Can only be used by moderators.
command!(faq_add(_context, message, _args, name: String, faq: String) {
    let guild_id = message.guild_id().unwrap();
    // Reject if they don't use quotes, since the faq wouldn't be added correctly otherwise
    if message.content_safe().matches("\"").count() != 4 || name.is_empty() || faq.is_empty() {
        send_error_embed_or_say(&message, &format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help faq-add``` for info on how to format this command.",
                                        get_prefix_for_guild(&guild_id)));
        return Err(String::from("Could not add due to missing quotes or invalid args."));
    }
    let mut parsed_json = get_faq_json(&guild_id, &message);
    let name = name.to_lowercase();

    if !parsed_json.has_key(name.as_str()) {

        // Add the entry to the json
        let file = message.attachments.get(0);
        if let Some(image) = file {
            let mut array = JsonValue::new_array();
            let _ = array.push(faq.clone());
            let _ = array.push(image.url.clone());
            parsed_json[&name] = array;
        } else { //No attachment to the message
            let mut array = JsonValue::new_array();
            let _ = array.push(faq.clone());
            parsed_json[&name] = array;
        }

        // Write it back to the file
        write_faq_json(parsed_json, &guild_id);

        // Report success
        if let Err(_) = send_success_embed(&message, &format!("Added FAQ `{}`. \nContents are:\n{}", name, faq)) {
            say_into_chat(&message, &format!("Added FAQ `{}`. \nContents are:\n{}", name, faq));
        }
    } else {
        send_error_embed_or_say(&message, "Cannot add, dictionary already contains an entry for that name.
                                 Try using ```faq-set``` instead, or removing it.");
        return Err(String::from("Could not add due to an already existing key for provided faq."));
    }
});

/// Retrieves a faq from the storage of the bot. {{{1
command!(faq_get(_context, message, _args) {
    let _ = message.channel_id.broadcast_typing();
    let parsed_json = get_faq_json(&message.guild_id().unwrap(), &message);
    let server_prefix = get_prefix_for_guild(&message.guild_id().unwrap());
    let mut request = String::new();

    // Correctly fix message based on which alias they use
    if message.content_safe().starts_with(format!("{}faq", server_prefix).as_str()) {
        request = fix_message(message.content_safe(), "faq");
    } else if message.content_safe().starts_with(format!("{}faw", server_prefix).as_str()) {
        request = fix_message(message.content_safe(), "faw");
    }

    if request.is_empty() {
        // Call the other command's function, since the user is looking for a list
        return faqs(_context, message, _args);
    } else if parsed_json.is_empty() {
        send_error_embed_or_say(&message, "Sorry, no FAQs configured.");
        return Err(String::from("No FAQs configured, cannot pick one."));
    }

    // Make lowercase
    request = request.to_lowercase();

    // Create a list of just keys
    let faq_key_list: Vec<&str> = parsed_json.entries().map(|(key, _value)| {
        key
    }).collect();

    // Find the closest match to what they asked for
    let (dist, closest_match) = get_closest_match(&faq_key_list, &request);

    if dist > 0 && dist <= DISTANCE_SENSITIVITY {
        say_into_chat(&message, &format!("I didn't find `{}`, but I did find the next closest FAQ, `{}`:", request, closest_match));
    } else if dist > DISTANCE_SENSITIVITY {
        send_error_embed_or_say(&message, &format!("I didn't find `{}`, and no other FAQ was similar enough.", request));
        return Err(String::from("FAQ distance was too great from request, failing out..."));
    }

    // Determine if the message also has an associated image
    if parsed_json[closest_match].len() == 2 {
        // Build message
        if let Err(_) = message.channel_id.send_message(|a| a
                                                        .embed(|b| b
                                                               .title(closest_match)
                                                               .description(parsed_json[closest_match][0].as_str().unwrap())
                                                               .image(parsed_json[closest_match][1].as_str().unwrap())
                                                               .color(Colour::from_rgb(119,0,255))
                                                               .timestamp(message.timestamp.to_rfc3339())
                                                              )) {
            say_into_chat(&message, format!("FAQ for `{}`:\n```{}```\n(Normally there would be an image here.)", request, parsed_json[closest_match][0].as_str().unwrap()));
        }

    } else if parsed_json[closest_match].len() == 1 {
        // Build message
        if let Err(_) = message.channel_id.send_message(|a| a
                                                        .embed(|b| b
                                                               .title(closest_match)
                                                               .description(parsed_json[closest_match][0].as_str().unwrap())
                                                               .color(Colour::from_rgb(119,0,255))
                                                               .timestamp(message.timestamp.to_rfc3339())
                                                              )) {
            say_into_chat(&message, format!("FAQ for `{}`:\n```{}```", request, parsed_json[closest_match][0].as_str().unwrap()));
        }
    }
});

/// Deletes a stored faq. Administrators only. {{{1
command!(faq_delete(_context, message) {
    let guild_id = message.guild_id().unwrap();
    let mut parsed_json = get_faq_json(&guild_id, &message);
    let request = fix_message(message.content_safe(), "faq-delete");
    let request = request.replace("\"", ""); //Remove quotes if they used any

    // Check if the faq exists
    if !parsed_json.has_key(&request) {
        send_error_embed_or_say(&message, &format!("Sorry, I didn't find anything for `{}`. It might've been already deleted.", request));
    } else { // Key is found
        // Do the deletion
        let _ = parsed_json.remove(&request);
        write_faq_json(parsed_json, &guild_id);

        // Build message
        if let Err(_) = send_success_embed(&message, &format!("Success, faq for `{}` was deleted.", request)) {
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
    let guild_id = message.guild_id().unwrap();
    if message.content_safe().matches("\"").count() != 4 || name.is_empty() || faq.is_empty() {
        send_error_embed_or_say(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help faq-set``` for info on how to format this command.",
                                        get_prefix_for_guild(&guild_id)
                                        ).as_str());
        return Err(String::from("Could not set faq due to invalid args."));
    } else {
        let mut parsed_json = get_faq_json(&guild_id, &message);
        let name = name.to_lowercase();

        if parsed_json.has_key(name.as_str()) {

            // Modify the entry
            let file = message.attachments.get(0);
            if let Some(image) = file {
                let mut array = JsonValue::new_array();
                // Add the text and image
                let _ = array.push(faq.clone());
                let _ = array.push(image.url.clone());
                parsed_json[&name] = array;
            } else {
                let mut array = JsonValue::new_array();
                let _ = array.push(faq.clone());
                parsed_json[&name] = array;
            }

            // Write it back to the file
            write_faq_json(parsed_json, &guild_id);

            if let Err(_) = send_success_embed(&message, format!("Success, set faq `{}` for concept `{}`.", faq, name).as_str()) {
                say_into_chat(&message, format!("Success, set faq `{}` for concept `{}`.", faq, name));
            }
        } else {
            send_error_embed_or_say(&message, "Cannot set, key not found in dictionary. Try using ```faq add``` instead.");
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
        log_error!(
            "Error writing to json file, aborting with error: {:?}",
            error
        );
    } else {
        log_info!("Wrote to json file: {}", faq_file);
    }
}

/// Takes a JsonValue, and returns a string of all the keys {{{1
/// in a comma seperated list.
fn jsonvalue_as_comma_list(json_value: &JsonValue) -> String {
    if json_value.is_empty() {
        return String::from("No FAQs configured.");
    }
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
    result.push('.'); // Add a period to end the list
    result
}

/// Opens the faqs file, and returns the Json object contained {{{1
/// within it. Returns a JsonValue
pub fn get_faq_json(guild: &GuildId, message: &Message) -> JsonValue {
    // Determine file name based on the guild in question
    let faq_file = format!("{:?}-faqs.json", guild);

    // Open the json file for writing, nuking any previous contents
    let file_result = OpenOptions::new().read(true).open(faq_file.as_str());

    match file_result { //If it errors here, it's probably because the file doesn't exist
        Err(_) => {
            let mut file_handle = File::create(faq_file).expect("Could not create faqs file.");
            file_handle.write_all(b"{}").expect(
                "Got error writing to newly created json file.",
            ); //Write empty json object to it

            return JsonValue::new_object(); //Return empty database
        }
        Ok(mut file) => {

            let mut data = String::new();
            file.read_to_string(&mut data).expect(
                "Something went wrong reading the faqs file.",
            );

            let data = data.trim(); //Remove the newline from the end of the string if present

            match json::parse(data) {
                Ok(result) => result,
                Err(_) => {
                    log_error!("Unable to parse json from faqs file.");
                    say_into_chat(
                        &message,
                        "Sorry, I couldn't read the database for this server.",
                    );
                    JsonValue::new_object()
                }
            }
        }
    }
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
        assert_eq!(listed, "first, second, third, fourth.");
    }
}

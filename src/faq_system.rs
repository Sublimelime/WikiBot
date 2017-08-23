extern crate serenity;
extern crate rand;
extern crate json;

use self::json::JsonValue;
use self::serenity::utils::Colour;
use self::serenity::model::*;

use common_funcs::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use constants::*;
use std::fs::OpenOptions;
use levenshtein::*;


/// A struct that holds all the data about a faq entry, including the channel and message.
struct FAQEntry {
    pub channel: ChannelId,
    pub message: Message,
    pub message_id: MessageId,
}

/// Retrieves a faq from the list. {{{1
command!(faq_get(_context, message) {
});

command!(faqs(_context, message) {
});

command!(faq_add(_context, message) {
});

command!(faq_delete(_context, message) {
});

command!(faq_deleteall(_context, message) {
});

command!(faq_set(_context, message) {
});

///Takes a JsonValue, and writes it to the ratios file. {{{1
pub fn write_faq_backup(value: JsonValue, guild: &GuildId) {
    // Determine file name based on the guild in question
    let ratio_file = format!("{:?}-faqs.json", guild);

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
        make_log_entry(format!("Wrote to json file: {}", ratio_file), "Info");
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

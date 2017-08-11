/// A file containing functions, and small commands

extern crate serenity;
extern crate rand;
extern crate json;

use self::json::*;
use self::rand::Rng;
use self::serenity::model::Message;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

command!(ping(_context, message) {
    let replies = ["Pong!", "Marco!", "Roger!", "I hear ya!",
    "Good day.", "Hello!", "What's up?","I'm alive!",
    "Hearing you loud and clear.","PiNg!", "Yep, still here.",
    "Nah, totally offline.", "Sup?", "What's a bot gotta do to get some sleep around here?",
    "Running.", "Uptime is my prime focus.", "It... It's not like I wanted to be pinged by you... :blush:",
    "!gniP", "*Snore* :zzz:"];
    let random = rand::thread_rng().gen_range(0, replies.len());

    let _ = message.reply(replies[random]);
});

command!(info(_context, message) {
    let mut reply = String::from("Author: Gangsir\nDesc: A simple bot that fetches information related to factorio.\nFor help, use ");
    reply.push_str(::PREFIX);
    reply.push_str("help. Thanks, and enjoy!");
    let _ = message.reply(&reply[..]);
});

command!(page(_context, message) {
    let mut final_message = String::from("https://wiki.factorio.com/");

    // Remove command from message content, and code-ify it
    let mut modified_content = fix_message(message.content_safe(), "page ");
    modified_content = modified_content.replace(" ", "_");

    final_message.push_str(&modified_content[..]); //add the specified page to the end

    // Post link back into chat
    say_into_chat(&message, &final_message[..]);
});


/// Returns info about a specified function or field of the mod api.
command!(modapi(_context, message) {

});

// Functions -----------------

/// Says a message into chat. Takes the Message object of the event,
/// and a str to say.
pub fn say_into_chat(message: &Message, speech: &str) {
    if let Err(error) = message.channel_id.say(speech) {
        println!(
            "[Error] Unable to send reply message: {}. Error is {}",
            speech,
            error
            );
    }
}

/// Corrects a message, removing the command from it, and truncating
/// everything after ||
pub fn fix_message(message: String, command: &str) -> String {

    let mut modified_content = message.replace(command, "").replace(::PREFIX, "");

    // Truncate message to ||
    if let Some(index) = modified_content.find(" ||") {
        modified_content.truncate(index as usize);
    }

    modified_content
}

/// Opens the ratios file, and returns the Json object contained
/// within it. Returns a JsonValue
pub fn get_ratio_json() -> JsonValue {
    let mut file = File::open("ratios.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).expect(
        "Something went wrong reading the ratios file.",
        );

    let data = data.trim(); //Remove the newline from the end of the string if present

    json::parse(data).expect("Unable to parse json from ratios file.")
}

///Takes a JsonValue, and writes it to the ratios file.
pub fn write_ratio_json(value: JsonValue) {
    // Open the json file for writing, nuking any previous contents
    let mut file = OpenOptions::new().write(true).truncate(true).open("ratios.json").expect("Unable to open json file for writing.");

    // Write json to file
    if let Err(error) = value.write(&mut file) {
        println!("Error writing to json file, aborting with error: {:?}", error);
    }
}


#[cfg(test)]
mod tests {
    use super::fix_message;
    #[test]
    fn test_message_fixing() {
        let result = fix_message(String::from("+command This is a test message"), "command ");
        assert_eq!(result, "This is a test message");

        let result = fix_message(
            String::from("+command arg || Some random chat text"),
            "command ",
            );
        assert_eq!(result, "arg");
    }
}

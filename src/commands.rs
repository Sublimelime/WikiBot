extern crate inflector;
extern crate serenity;
extern crate rand;
extern crate json;

use self::inflector::Inflector;
use self::json::*;
use self::rand::Rng;
use self::serenity::model::Message;
use std::fs::File;
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

    let mut modified_content = fix_message(message.content_safe(), "page ");

    // Remove command from message content, and code-ify it
    modified_content = modified_content.replace(" ", "_");

    final_message.push_str(&modified_content[..]); //add the specified page to the end

    // Post link back into chat
    say_into_chat(message, &final_message[..]);
});

/// Prints out a grand list of all current stored ratios.
command!(ratios(_context, message) {
    let parsed_json = get_ratio_json();

    println!("{:?}", parsed_json["code"].as_number().unwrap().as_parts().1);
});

/// Adds a ratio to the list of current ratios.
/// Can only be used by moderators.
command!(ratio_add(_context, message) {
    let parsed_json = get_ratio_json();
});

/// Retrieves a ratio from the storage of the bot.
command!(ratio_get(_context, message) {
    let parsed_json = get_ratio_json();

});

/// Returns info about a specified function or field of the mod api.
command!(modapi(_context, message) {

});

// Functions -----------------

/// Says a message into chat. Takes the Message object of the event,
/// and a str to say.
fn say_into_chat(message: &Message, speech: &str) {
    if let Err(error) = message.channel_id.say(speech) {
        println!("[Error] Unable to send reply message: {}. Error is {}", speech, error);
    }
}

/// Corrects a message, removing the command from it, and truncating
/// everything after ||
fn fix_message(message: String, command: &str) -> String {

    let mut modified_content = message.replace(command, "").replace(::PREFIX,"");

    // Truncate message to ||
    if let Some(index) = modified_content.find(" ||") {
        modified_content.truncate(index as usize);
    }

    modified_content
}

/// Opens the ratios file, and returns the Json object contained
/// within it. Returns a JsonValue
fn get_ratio_json() -> JsonValue {
    let mut file = File::open("ratios.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).expect(
        "Something went wrong reading the ratios file.",
        );

    let data = data.trim(); //Remove the newline from the end of the string if present

    json::parse(data).expect("Unable to parse json from ratios file.")
}

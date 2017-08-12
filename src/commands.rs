/// A file containing functions, and small commands

extern crate serenity;
extern crate rand;
extern crate json;

use self::json::*;
use self::rand::Rng;
use std::process::Command;
use self::serenity::model::Message;
use std::fs::{File, OpenOptions};
use std::fmt::Display;
use std::io::prelude::*;

/// Your standard ping command, replies with random replies
command!(ping(_context, message) {
    let replies = ["Pong!", "Marco!", "Roger!", "I hear ya!",
    "Good day.", "Hello!", "What's up?","I'm alive!",
    "Hearing you loud and clear.","PiNg!", "Yep, still here.",
    "Nah, totally offline.", "Sup?", "What's a bot gotta do to get some sleep around here?",
    "Running.", "Uptime is my prime focus.", "It... It's not like I wanted to be pinged by you... :blush:",
    "!gniP", "*Snore* :zzz:", "Cheers!", "Yes?"];
    let random = rand::thread_rng().gen_range(0, replies.len());

    let _ = message.reply(replies[random]);
});

/// Prints various info about the bot itself.
command!(info(_context, message) {
    let mut reply = String::from("Author: Gangsir\nDesc: A simple bot that fetches information related to factorio.\nFor help and a list of commands, use ");
    reply.push_str(::PREFIX);
    reply.push_str("help. Thanks, and enjoy!");
    let _ = message.reply(&reply[..]);
});

/// Prints current system status, including uptime
/// of the bot into chat.
command!(uptime(_context, message) {
    if let Ok(result) = Command::new("uptime").output() {
        let _ = message.reply(String::from_utf8(result.stdout).unwrap().as_str());
    } else {
        println!("Unable to run uptime command.")
    }
});

/// Prints data about the host into the chat.
command!(host(_context, message) {
    use std::env::consts::*;
    let mut uptime = String::new();

    if let Ok(result) = Command::new("uptime").arg("-p").output() {
        uptime = String::from_utf8(result.stdout).unwrap();
    } else {
        println!("Unable to run uptime command.")
    }
    let _ = message.reply(format!("Host OS: {}\nHost Arch: {}\nCurrent uptime: {}
                                  \n Powered by Rust(stable). https://rust-lang.org",
                                  OS,
                                  ARCH,
                                  uptime
                                  ).as_str());
});

/// Links a page on the wiki.
command!(page(_context, message) {
    let mut final_message = String::from("https://wiki.factorio.com/");

    // Remove command from message content, and code-ify it
    let mut modified_content = String::new();
    if message.content_safe().starts_with(format!("{}page", ::PREFIX).as_str()) {
        modified_content = fix_message(message.content_safe(), "page ");
    } else if message.content_safe().starts_with(format!("{}link", ::PREFIX).as_str()) {
        modified_content = fix_message(message.content_safe(), "link ");
    }

    modified_content = modified_content.replace(" ", "_");

    final_message.push_str(modified_content.as_str()); //add the specified page to the end

    // Post link back into chat
    say_into_chat(&message, final_message);
});

// Functions -----------------

/// Says a message into chat. Takes the Message object of the event,
/// and a str to say.
pub fn say_into_chat<T>(message: &Message, speech: T)
where
    T: Display,
{
    if let Err(error) = message.channel_id.say(format!("{}", speech).as_str()) {
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

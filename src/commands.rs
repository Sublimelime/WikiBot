/// A file containing functions, and small commands

extern crate serenity;
extern crate rand;
extern crate json;
extern crate chrono;

use self::json::*;
use self::rand::Rng;
use std::process::Command;
use self::serenity::model::Message;
use self::serenity::utils::Colour;
use std::fs::File;
use std::fmt::Display;
use std::io::prelude::*;
use constants::*;
use self::chrono::{Utc as UTC, DateTime};

/// Your standard ping command, replies with random replies
command!(ping(_context, message) {
    let replies = ["Pong!", "Marco!", "Roger!", "I hear ya!",
    "Good day.", "Hello!", "What's up?","I'm alive!",
    "Hearing you loud and clear.","PiNg!", "Yep, still here.",
    "Nah, totally offline.", "Sup?", "What's a bot gotta do to get some sleep around here?",
    "Running.", "Uptime is my prime focus.", "It... It's not like I wanted to be pinged by you... :blush:",
    "!gniP", "*Snore* :zzz:", "Cheers!", "Yes?"];
    let random = rand::thread_rng().gen_range(0, replies.len());

    reply_into_chat(&message, replies[random]);
});

/// Prints various info about the bot itself.
command!(info(_context, message) {
    let mut reply = String::from("A simple bot that fetches information related to factorio.\nFor help and a list of commands, use ");
    reply.push_str(get_prefix_for_guild(&message).as_str());
    reply.push_str("help. All commands that do not take arguments support talking after the commands with ||, commands that take arguments support it if it says so in help.
                   \nThanks, and enjoy! For info about the host of this bot, run the `host` command.");
    let _ = message.channel_id.send_message(|a| a
                                            .embed(|b| b
                                                   .title("WikiBot")
                                                   .description(reply.as_str())
                                                   .field(|c| c
                                                          .name("Author")
                                                          .value("Gangsir")
                                                          )
                                                   .field(|c| c
                                                          .name("Contact")
                                                          .value("I'm in the factorio discord server, or you can reddit PM me [here](https://www.reddit.com/message/compose?to=Gangsir).")
                                                          )
                                                   .field(|c| c
                                                         .name("Programming language")
                                                         .value("Made in [Rust](https://rust-lang.org).")
                                                          )
                                                   .field(|c| c
                                                         .name("Library")
                                                         .value("Made with [Serenity](https://crates.io/crates/serenity).")
                                                          )
                                                   .timestamp(message.timestamp.to_rfc3339())
                                                   .color(Colour::from_rgb(255, 255, 255))
                                                   )
                                            );
});

/// Prints current system status, including uptime
/// of the bot into chat.
command!(uptime(_context, message) {
    if let Ok(result) = Command::new("uptime").output() {
        reply_into_chat(&message, String::from_utf8(result.stdout).unwrap());
    } else {
        make_log_entry("Unable to run uptime command.".to_owned(), "Error");
    }
});

/// Prints data about the host into the chat.
command!(host(_context, message) {
    use std::env::consts::*;
    let mut uptime = String::new();

    if let Ok(result) = Command::new("uptime").arg("-p").output() {
        uptime = String::from_utf8(result.stdout).unwrap();
    } else {
        make_log_entry("Unable to run uptime command.".to_owned(), "Error");
    }
    let embed_result = message.channel_id.send_message(|a| a
                                                       .embed(|b| b
                                                              .description("Powered by [Rust Stable](https://rust-lang.org).")
                                                              .field(|c| c
                                                                     .name("Host OS:")
                                                                     .value(OS))
                                                              .field(|c| c
                                                                     .name("Host processor arch:")
                                                                     .value(ARCH))
                                                              .field(|c| c
                                                                     .name("Uptime:")
                                                                     .value(uptime.as_str()))
                                                              .timestamp(message.timestamp.to_rfc3339())
                                                              .color(Colour::from_rgb(255,255,255))
                                                             ));

    if let Err(_) = embed_result {
        reply_into_chat(&message, format!("Host OS: {}\nHost Arch: {}\nCurrent uptime: {}
                                  \n Powered by Rust(stable). https://rust-lang.org",
                                  OS,
                                  ARCH,
                                  uptime
                                  ));
    }
});

/// Links a page on the wiki.
command!(page(_context, message) {
    let mut final_message = String::from("https://wiki.factorio.com/");

    // Remove command from message content, and code-ify it
    let mut modified_content = String::new();
    if message.content_safe().starts_with(format!("{}page", get_prefix_for_guild(&message)).as_str()) {
        modified_content = fix_message(message.content_safe(), "page ", &message);
    } else if message.content_safe().starts_with(format!("{}link", get_prefix_for_guild(&message)).as_str()) {
        modified_content = fix_message(message.content_safe(), "link ", &message);
    }

    modified_content = modified_content.replace(" ", "_");

    final_message.push_str(modified_content.as_str()); //add the specified page to the end

    // Post link back into chat
    say_into_chat(&message, final_message);
});

command!(fff_old(_context, message) {
    let mut final_message = String::from("https://factorio.com/blog/post/fff-");

    let mut modified_content = String::new();

    if message.content_safe().starts_with(format!("{}fff-old", get_prefix_for_guild(&message)).as_str()) {
        modified_content = fix_message(message.content_safe(), "fff-old ", &message);
    } else if message.content_safe().starts_with(format!("{}blog-old", get_prefix_for_guild(&message)).as_str()) {
        modified_content = fix_message(message.content_safe(), "blog-old ", &message);
    }

    final_message.push_str(modified_content.as_str());
    reply_into_chat(&message, final_message);
});


// Functions -----------------

/// Makes a log entry, by prepending the time and date of the entry to what's
/// provided to the function.
pub fn make_log_entry(entry: String, kind: &str) {
    let timestamp: String = UTC::now().to_rfc3339();
    println!("[{} at {}] {}", kind, timestamp, entry);
}

/// Sends a simple error embed. Provide the reason for erroring.
pub fn send_error_embed(message: &Message, reason: &str) -> serenity::Result<Message> {
    message.channel_id.send_message(|a| {
        a.embed(|e| {
            e.title("Error")
                .description(reason)
                .timestamp(message.timestamp.to_rfc3339())
                .color(Colour::from_rgb(255, 0, 0))
        })
    })
}


/// Replies a message into chat, pinging the original summoner.
pub fn reply_into_chat<T>(message: &Message, speech: T)
where
    T: Display,
{
    if let Err(error) = message.reply(format!("{}", speech).as_str()) {
        make_log_entry(
            format!(
                "Unable to send reply message: {}. Error is {}",
                speech,
                error
            ),
            "Error",
        );
    }
}

/// Says a message into chat. Takes the Message object of the event,
/// and a str to say.
pub fn say_into_chat<T>(message: &Message, speech: T)
where
    T: Display,
{
    if let Err(error) = message.channel_id.say(format!("{}", speech).as_str()) {
        make_log_entry(
            format!(
                "Unable to send reply message: {}. Error is {}",
                speech,
                error
            ),
            "Error",
        );
    }
}

/// Corrects a message, removing the command from it, and truncating
/// everything after ||
pub fn fix_message(message: String, command: &str, msg: &Message) -> String {

    let mut modified_content = message.replace(command, "").replace(get_prefix_for_guild(&msg).as_str(), "");

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

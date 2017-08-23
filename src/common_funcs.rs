/// A file containing functions, and small commands

extern crate serenity;
extern crate json;
extern crate chrono;

use self::json::*;
use self::serenity::model::{Message, GuildId};
use self::serenity::utils::Colour;
use std::fs::{File, OpenOptions};
use std::fmt::Display;
use std::io::prelude::*;
use constants::*;
use self::chrono::Utc as UTC;

/// Makes a log entry, by prepending the time and date of the entry to what's {{{1
/// provided to the function.
pub fn make_log_entry(entry: String, kind: &str) {
    let timestamp: String = UTC::now().to_rfc3339();
    println!("[{} at {}] {}", kind, timestamp, entry);
}

/// Sends a simple error embed. Provide the reason for erroring. {{{1
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

/// Sends a simple success embed, with provided description. {{{1
pub fn send_success_embed(message: &Message, reason: &str) -> serenity::Result<Message> {
    message.channel_id.send_message(|a| {
        a.embed(|e| {
            e.title("Success")
                .description(reason)
                .timestamp(message.timestamp.to_rfc3339())
                .color(Colour::from_rgb(0, 255, 0))
        })
    })
}


/// Replies a message into chat, pinging the original summoner. {{{1
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

/// Says a message into chat. Takes the Message object of the event, {{{1
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

/// Corrects a message, removing the command from it, and truncating {{{1
/// everything after ||
pub fn fix_message(message: String, command: &str, prefix: &str) -> String {
    let mut modified_content = message;

    // Truncate message to ||
    if let Some(index) = modified_content.find(" ||") {
        modified_content.truncate(index as usize);
    }

    // Remove the command from it, and prefix
    modified_content = modified_content.replace(command, "").replace(prefix, "");

    // Remove the bot's name, if used via mention
    modified_content = modified_content.replace(format!("@{}", BOT_NAME).as_str(), "");

    //Trim spaces off
    modified_content = modified_content.trim().to_owned();

    modified_content
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

            let result = json::parse(data);

            if let Err(_) = result {
                make_log_entry("Unable to parse json from faqs file.".to_owned(), "Error");
                say_into_chat(
                    &message,
                    "Sorry, I couldn't read the database for this server.",
                );
                return JsonValue::new_object();
            } else {
                return result.unwrap();
            }
        }
    }
}

// Tests {{{1
#[cfg(test)]
mod tests {
    use super::fix_message;
    #[test]
    fn can_fix_messages_with_arg() {
        let message = String::from("+faqs get steam");
        assert_eq!("steam", fix_message(message, "faqs get ", "+"))
    }

    #[test]
    fn can_fix_messages_without_arg() {
        let message = String::from("+faqs get");
        assert_eq!("", fix_message(message, "faqs get", "+"))
    }

    #[test]
    fn can_fix_messages_with_pipes() {
        let message = String::from("+faqs get steam || Hello, I'm talking after this.");
        assert_eq!("steam", fix_message(message, "faqs get", "+"))
    }

    #[test]
    fn can_fix_messages_without_arg_and_with_pipes() {
        let message = String::from("+faqs get || Comprehensive test coverage!");
        assert_eq!("", fix_message(message, "faqs get", "+"))
    }
}

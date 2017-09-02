/// A file containing functions, and small commands

use serenity;
use serenity::model::Message;
use serenity::utils::Colour;

use std::fmt::Display;
use std::collections::BTreeMap;

use constants::*;
use levenshtein::*;

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
        log_error!(
            "Unable to send reply message: {}. Error is {}",
            speech,
            error
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
        log_error!(
            "Unable to send reply message: {}. Error is {}",
            speech,
            error
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


/// Takes a slice of strs and a request, returns the distance and the {{{1
/// str that is the closest match. Makes the assumption that list will never
/// be empty.
pub fn get_closest_match<'a>(list: &[&'a str], request: &'a str) -> (usize, &'a str) {
    // Short circut if there's an exact match
    if let Some(found) = list.iter().find(|&&p| p == request) {
        return (0, found);
    }
    // Build a ranked list of close matches
    let mut possiblities = BTreeMap::new();

    for entry in list.iter() {
        possiblities.insert(levenshtein(entry, request), entry);
    }

    //return the smallest distance key, unwrap because there should be a value
    let (dist, smallest) = possiblities.iter_mut().next().unwrap();
    return (*dist, smallest);
}

// Tests {{{1
#[cfg(test)]
mod tests {
    use super::*;

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
    fn closest_match_with_exact() {
        let request = "beacon";
        let list = vec!["wiki", "beacon", "twelve", "blah", "test"];
        let (dist, closest_match) = get_closest_match(&list, request);
        assert_eq!(closest_match, "beacon");
        assert_eq!(dist, 0);
    }

    #[test]
    fn closest_match_with_slightly_incorrect() {
        let request = "baecon";
        let list = vec!["wiki", "beacon", "twelve", "blah", "test"];
        let (dist, closest_match) = get_closest_match(&list, request);
        assert_eq!(closest_match, "beacon");
        assert_eq!(dist, 2);
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

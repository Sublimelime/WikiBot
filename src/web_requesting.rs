/// A file used to hold all commands that make web requests.

extern crate atom_syndication;
extern crate serenity;
extern crate reqwest;

use commands::*;
use std::thread;
use self::atom_syndication::Feed;
use self::serenity::model::Message;
use self::serenity::utils::Colour;
use self::serenity::Error;
use std::io::BufReader;

/// Requests a link to the newest FFF posted by the devs.
command!(fff(_context, msg) {
    // Indicate command might take a bit
    let _ = msg.channel_id.broadcast_typing();
    let fail_message = "Sorry, I couldn't get a response from Wube's RSS site.";
    let fail_message_rss = "Unable to parse RSS obtained from blog RSS site.";

    // Clone message, since the original message doesn't stay alive after this function, so
    // can't be passed to the closure
    let message = msg.clone();
    thread::spawn(move || {
        // Get rss channel from wube's blog rss url
        if let Ok(response) = reqwest::get("https://www.factorio.com/blog/rss") {
            if response.status().is_success() {
                // Change response into a buffered reader
                let reader = BufReader::new(response);
                // Parse the RSS
                if let Ok(feed) = Feed::read_from(reader) {
                    let latest_entry = feed.entries().first().unwrap();
                    let update_time = latest_entry.updated();
                    if let Some(link) = latest_entry.links().first() {
                        // Make embed to show results
                        if let Err(error) = send_fff_embed(&message, update_time, link.href()) {
                            println!("Got error sending embed for fff results, {}", error);
                            reply_into_chat(&message, "Sorry, I was unable to send the results as an embed. Instead, have them plain:");
                            say_into_chat(&message, format!("Latest FFF as of roughly {}:\n{}", update_time, link.href()).as_str());
                        }
                    } else {
                        reply_into_chat(&message, fail_message_rss);
                    }
                } else {
                    reply_into_chat(&message, fail_message_rss);
                }
            } else {
                reply_into_chat(&message, fail_message);
            }
        } else {
            reply_into_chat(&message, fail_message);
        }
    });
});

/// Sends the result of the RSS get in an embed. Extracted for code simplicity.
fn send_fff_embed(message: &Message, update_time: &str, link: &str) -> Result<Message, Error> {
    message.channel_id.send_message(|a| {
        a.embed(|e| {
            e.description("FFF Results:")
                .field(|f| {
                    f.name(format_rss_time(update_time).as_str()).value(link)
                })
                .timestamp(message.timestamp.to_rfc3339())
                .color(Colour::from_rgb(200, 100, 10))
        })
    })
}

/// Formats the time we get from an RSS feed into a more pleasant format
/// Basically truncates the time from the end, and a few other changes
fn format_rss_time(time: &str) -> String {
    let mut result = String::from(time);
    // Truncate message to date only
    if let Some(index) = time.find("T") {
        result.truncate(index as usize);
    }
    format!("Date: {}", result)
}

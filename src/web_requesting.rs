/// A file used to hold all commands that make web requests.

extern crate atom_syndication;
extern crate serenity;
extern crate reqwest;

use common_funcs::*;
use std::thread;
use self::atom_syndication::Feed;
use self::serenity::model::Message;
use self::serenity::utils::Colour;
use self::serenity::Error;
use std::io::{BufReader, Read};

/// Requests a link to the newest FFF posted by the devs. {{{1
command!(fff(_context, msg) {
    // Indicate command might take a bit
    let _ = msg.channel_id.broadcast_typing();
    let fail_message = "Sorry, I couldn't get a response from Wube's RSS site.";
    let fail_message_rss = "Unable to parse RSS obtained from blog RSS site.";

    // Clone message, since the original message doesn't stay alive after this function, so
    // can't be passed to the closure
    let message = msg.clone();

    //Spawn thread to handle getting the results {{{2
    thread::spawn(move || {
        // Get rss channel from wube's blog rss url
        if let Ok(response) = reqwest::get("https://www.factorio.com/blog/rss") {
            if response.status().is_success() {
                // Change response into a buffered reader
                let reader = BufReader::new(response);
                // Parse the RSS
                if let Ok(feed) = Feed::read_from(reader) {

                    // Data about the entry
                    let latest_entry = feed.entries().first().unwrap();
                    let update_time = latest_entry.updated();

                    if let Some(link) = latest_entry.links().first() {
                        // Make embed to show results
                        let link = link.href();

                        // Pull number off of the last three digits of the url
                        let number: u32 = link[link.len() - 3 ..].parse().unwrap();

                        // Send the FFF embed detailing it
                        if let Err(error) = send_fff_embed(&message, update_time, link, number) {
                            make_log_entry(format!("Got error sending embed for fff results, {}", error), "Error");
                            reply_into_chat(&message, "Sorry, I was unable to send the results as an embed. Instead, have them plain:");
                            say_into_chat(&message, format!("Latest FFF as of roughly {}:\n{}", update_time, link).as_str());
                        }

                    } else {
                        let _ = send_error_embed(&message, fail_message_rss);
                    }
                } else {
                    let _ = send_error_embed(&message, fail_message_rss);
                }
            } else {
                let _ = send_error_embed(&message, fail_message);
            }
        } else {
            let _ = send_error_embed(&message, fail_message);
        }
    });
});

/// Requests the latest version from Wube's servers. {{{1
command!(version(_context, msg) {
    // Indicate command might take a bit
    let _ = msg.channel_id.broadcast_typing();
    let fail_message = "Sorry, I couldn't get a response from Wube's version site.";

    // Clone message, since the original message doesn't stay alive after this function, so
    // can't be passed to the closure
    let message = msg.clone();

    //Spawn thread to handle getting the results {{{2
    let _ = thread::spawn(move || {
        // Vars to hold the results
        let mut latest_stable = String::new();
        let mut latest_experimental = String::new();

        // Make request
        if let Ok(mut response) = reqwest::get("https://factorio.com") {
            if response.status().is_success() {
                let mut html = String::new();
                response.read_to_string(&mut html).unwrap();
                // Horrible hack to find latest version
                // FIXME: Ask hanziq about getting latest versions without auth
                if let Some(loc) = html.find("Stable:") {
                    latest_stable = get_stable_version_string(loc, &html);
                }
                if let Some(loc) = html.find("Experimental:") {
                    latest_experimental = get_experimental_version_string(loc, &html);
                }
            } else {
                say_into_chat(&message, fail_message);
            }
        } else {
            say_into_chat(&message, fail_message);
        }
        if !latest_stable.is_empty() && !latest_experimental.is_empty() {
            if let Err(_) = send_version_embed(&message, &latest_stable, &latest_experimental) {
                make_log_entry("Unable to send an embed of the results of a version query.".to_owned(), "Error");
                say_into_chat(&message, format!("I got a result, but was unable to send an embed of the results.
                              \nInstead, have them plain:\n Latest Stable: {}\nLatest experimental: {}", latest_stable, latest_experimental));
            }
        } else {
            say_into_chat(&message, fail_message);
        }
    });
});

/// Sends an embed that details the latest stable and experimental versions {{{1
fn send_version_embed(
    message: &Message,
    stable: &String,
    experimental: &String,
) -> Result<Message, Error> {
    message.channel_id.send_message(|a| {
        a.embed(|e| {
            e.description("Latest version:")
                .field(|f| f.name("Stable").value(stable.as_str()))
                .field(|f| f.name("Experimental").value(experimental.as_str()))
                .timestamp(message.timestamp.to_rfc3339())
                .color(Colour::from_rgb(200, 100, 10))
        })
    })
}

/// Sends the result of the RSS get in an embed. Extracted for code simplicity. {{{1
fn send_fff_embed(
    message: &Message,
    update_time: &str,
    link: &str,
    number: u32,
) -> Result<Message, Error> {
    message.channel_id.send_message(|a| {
        a.embed(|e| {
            e.description("FFF Results:")
                .field(|f| {
                    f.name(
                        format!("{}: #{}", format_rss_time(update_time).as_str(), number).as_str(),
                    ).value(link)
                })
                .timestamp(message.timestamp.to_rfc3339())
                .color(Colour::from_rgb(200, 100, 10))
        })
    })
}

/// Formats the time we get from an RSS feed into a more pleasant format {{{1
/// Basically truncates the time from the end, and a few other changes
fn format_rss_time(time: &str) -> String {
    let mut result = String::from(time);
    // Truncate message to date only
    if let Some(index) = time.find("T") {
        result.truncate(index as usize);
    }
    format!("Date: {}", result)
}

/// Slices html location to find stable version. {{{1
fn get_stable_version_string(loc: usize, html: &str) -> String {
    html[loc + 8..loc + 15].to_owned()
}

/// Slices html location to find stable version. {{{1
fn get_experimental_version_string(loc: usize, html: &str) -> String {
    html[loc + 14..loc + 21].to_owned()
}

// Tests {{{1
#[cfg(test)]
mod tests {
    extern crate reqwest;
    extern crate atom_syndication;

    use super::*;
    use std::io::BufReader;
    use self::atom_syndication::Feed;

    // Tests if getting and parsing the blog rss still works. {{{2
    #[test]
    fn can_get_and_parse_blog_rss() {
        // Get rss object
        let response = reqwest::get("https://www.factorio.com/blog/rss").unwrap();
        let reader = BufReader::new(response);
        // Read into a parseable format
        let feed = Feed::read_from(reader).unwrap();
        let latest_entry = feed.entries().first().unwrap();
        let update_time = latest_entry.updated();
        let link = latest_entry.links().first().unwrap().href();
        let number: u32 = link[link.len() - 3..].parse().unwrap();

        assert!(link.contains("https://www.factorio.com/blog/post/fff-")); //Make sure link is valid
        assert!(number > 200); //It should be at least 200, since that's the latest number rounded down
        assert!(update_time.contains("2017")); //Makes sense, if the rss is up-to-date
    }

    // Tests if getting and parsing the html still works. {{{2
    // this should only fail if they change their main page html
    #[test]
    fn can_get_and_parse_current_version() {
        let mut response = reqwest::get("https://www.factorio.com").unwrap();
        let mut html = String::new();
        let mut latest_stable = String::new();
        let mut latest_experimental = String::new();
        response.read_to_string(&mut html).unwrap();
        if let Some(loc) = html.find("Stable:") {
            latest_stable = get_stable_version_string(loc, &html);
        } else {
            panic!("Couldn't find Stable: field in html.")
        }
        if let Some(loc) = html.find("Experimental:") {
            latest_experimental = get_experimental_version_string(loc, &html);
        } else {
            panic!("Couldn't find Experimental: field in html.")
        }

        assert_ne!(latest_stable, String::from(""));
        assert_ne!(latest_experimental, String::from(""));

        //Should find no html
        assert!(!latest_stable.contains("<"));
        assert!(!latest_experimental.contains("<"));
        assert!(!latest_stable.contains(">"));
        assert!(!latest_experimental.contains(">"));

    }
}

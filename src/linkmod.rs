extern crate serenity;
extern crate json;
extern crate reqwest;

use constants::*;
use common_funcs::*;
use levenshtein::*;
use self::json::JsonValue;
use std::io::Read;
use self::serenity::model::Message;

/// Struct used to hold a bunch of data about a mod, for easy passing {{{1
#[derive(Debug)]
struct Mod {
    pub creation_date: String,
    pub last_updated: String,
    pub name: String,
    pub author: String,
    pub link: String,
    pub download_count: u64,
    pub source_path: Option<String>,
    pub homepage: Option<String>,
    pub summary: String,
    pub title: String,
    pub tag: Option<String>,
}

/// Creates an embed based on the recieved mod data. {{{1
/// Returns true if successful.
fn make_mod_embed(modification: Mod, message: &Message) -> bool {
    let result = message.channel_id.send_message(|a| a
                                                 .embed(|b| b
                                                        .description(&modification.summary)
                                                        .author(|c| c
                                                                .name(&modification.name)
                                                                .url(&modification.link))
                                                        .field(|c| c
                                                               .name("Author")
                                                               .value(&modification.author))
                                                       ));
    result.is_ok()
}

/// Turns a jsonValue into a Mod. Assumes this is a direct mod json entry. {{{1
/// This can be provided either by direct link, or by giving it one of the results
/// returned by a mod api search.
fn parse_json_into_mod(json: JsonValue) -> Mod {
    Mod {
        creation_date: format!("{}", json["created_at"]),
        last_updated: format!("{}", json["updated_at"]),
        name:  format!("{}", json["name"]),
        author:  format!("{}", json["owner"]),
        summary: format!("{}", json["summary"]),
        title: format!("{}", json["title"]),
        tag: None,
        download_count: json["download_count"].as_u64().unwrap_or(0),
        homepage: Some(format!("{}", json["homepage"])),
        source_path: Some(format!("{}", json["source_path"].as_str().unwrap_or("None"))),
        link: format!("https://mods.factorio.com/mods/{}/{}", json["owner"], json["name"])
    }
}

/// Makes a request, either returning empty, or the successful mod json {{{1
fn make_request(request: String, should_search: bool) -> JsonValue {
    let response;
    if should_search {
        response = reqwest::get(format!("https://mods.factorio.com/api/mods?q={}", request).as_str());
    } else {
        response = reqwest::get(format!("https://mods.factorio.com/api/mods/{}", request).as_str());
    }

    if let Ok(mut response) = response {
        if response.status().is_success() {
            let mut json = String::new();
            let _ = response.read_to_string(&mut json);
            return json::parse(&json).unwrap_or(JsonValue::new_object());
        } else {
            return JsonValue::new_object();
        }
    } else {
        return JsonValue::new_object();
    }
}

/// Returns true if the provided query is a real mod. {{{1
fn is_real_mod(request: &String) -> bool {
    let response = reqwest::get(format!("https://mods.factorio.com/api/mods/{}", request).as_str());

    if let Ok(mut response) = response {
        if response.status().is_success() {
            let mut json = String::new();
            let _ = response.read_to_string(&mut json);
            let json_result = json::parse(&json).unwrap_or(JsonValue::new_object());
            // It should only have the detail field if it's not a real mod.
            if json_result["detail"].is_null() {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
}

/// Links a mod into chat based on args. {{{1
command!(linkmod(_context, message) {
    let request = fix_message(message.content_safe(), "linkmod", &get_prefix_for_guild(&message.guild_id().unwrap()));
    let mut returned;

    if is_real_mod(&request) {
        // The request was a real mod, so we just need to retrieve it's info and make an embed.
        returned = make_request(request, false);
        if !returned.is_empty() {
            if make_mod_embed(parse_json_into_mod(returned), &message) {

            } else {
                say_into_chat(&message, "Couldn't get a response back from the mod portal.");
                return Err(String::from("Failed to get a response back from the mod portal."));
            }
        } else {
            say_into_chat(&message, "Couldn't get a response back from the mod portal.");
            return Err(String::from("Failed to get a response back from the mod portal."));
        }
    } else {
        // At this point, returned will be an object of pagination and an array of results.
        // We want to parse each result for their name only, and list them as possiblilities.
        // Since we just want to list them, we only need their name
        returned = make_request(request, true);
    }
});

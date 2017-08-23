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
fn parse_json_into_mod(json: &JsonValue) -> Mod {
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
fn make_request(request: &String) -> JsonValue {
    let response = reqwest::get(format!("https://mods.factorio.com/api/mods?q={}", request).as_str());

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

/// Makes an embed of search results. Takes a json array, and returns true {{{1
/// if able to make an embed of all the results
fn make_search_results_embed(message: &Message, results: JsonValue) -> bool {
    let result = message.channel_id.send_message(|a|
                                                 a.embed(|b| b
                                                         .title("Search results:")
                                                         .description(&serialize_search_results(&results))
                                                        ));
    if let Err(_) = result {
        false
    } else {
        true
    }
}

/// Takes a json array as input, returns a string of the search results serialized. {{{1
fn serialize_search_results(results: &JsonValue) -> String {
    let mut final_string = String::new();
    for (counter, entry) in results.members().enumerate() {
        if counter > 10 {
            break; //Don't put more than ten entries
        }
        final_string += format!("[{}](https://mods.factorio.com/{}/{}) by {}\n",
                                entry["title"],
                                entry["owner"],
                                entry["name"],
                                entry["owner"]).as_str();
    }
    final_string
}

/// Links a mod into chat based on args. {{{1
command!(linkmod(_context, message) {
    let request = fix_message(message.content_safe(), "linkmod", &get_prefix_for_guild(&message.guild_id().unwrap()));

    let returned = make_request(&request);
    if !returned.is_empty() {
        let returned_results = &returned["results"];

        if !returned_results.is_null() && !returned_results.is_empty() {
            for entry in returned_results.members() {
                let modification = parse_json_into_mod(&entry);

                // Check if the match is close enough, both on the
                // internal name and the title
                if levenshtein(&modification.name, &request) <= 3
                    || levenshtein(&modification.title, &request) <= 3 {
                        if !make_mod_embed(modification, &message) {
                            say_into_chat(&message, "Unable to make an embed here.");
                            return Err(String::from("Couldn't make an embed."));
                        } else {
                            return Ok(());
                        }
                    }
            }
            // At this point, it hasn't found an exact match,
            // so let's just make an embed with all the results it found
            if !make_search_results_embed(&message, returned_results.clone()) {

                say_into_chat(&message, "Unable to make an embed of search results here.");
                return Err(String::from("Couldn't make an embed of search results."));
            }
            return Ok(());
        } else {
            say_into_chat(&message, "There are no results for that query.");
            return Err(String::from("Didn't find any results for the request."));
        }
    } else {
        say_into_chat(&message, "Couldn't get a response back from the mod portal.");
        return Err(String::from("Failed to get a response back from the mod portal."));
    }
});

use json::{self, JsonValue};

use reqwest;

use serenity::model::Message;
use serenity::utils::Colour;

use std::io::Read;

use common_funcs::*;
use constants::*;
use levenshtein::*;

/// Structs used to hold a bunch of data about a mod/modder, for easy passing {{{1
#[derive(Debug, Clone)]
struct Mod {
    pub creation_date: String,
    pub last_updated: String,
    pub thumb: String, //The thumb image of the mod
    pub name: String, //Internal name of the mod
    pub author: String,
    pub link: String,
    pub download_count: u64,
    pub latest_version: String,
    pub factorio_version: String, //Latest factorio version the mod supports
    pub source_path: String,
    pub homepage: String,
    pub dependencies: String,
    pub summary: String,
    pub title: String, //Pretty title of the mod
    pub tag: Option<String>, //What tag the mod has
    pub download_link: String, //A direct download link to the newest version
}

#[derive(Debug)]
struct Modder {
    pub username: String,
    pub total_downloads: u64,
    pub latest_mod: Mod,
    pub last_updated_mod_date: String,
    pub first_mod: Mod,
    pub last_updated_mod: Mod,
}

/// Creates an embed based on the recieved mod data. {{{1
/// Returns true if successful.
fn make_mod_embed(modification: Mod, message: &Message) -> bool {
    let tag_str = modification.tag.as_ref().map(String::as_str).unwrap_or(
        "Not tagged.",
        );

    let result = message.channel_id.send_message(|a| {
        a.embed(|b| {
            b.description(&modification.summary)
                .author(|c| c.name(&modification.title).url(&modification.link))
                .thumbnail(&modification.thumb)
                .color(Colour::from_rgb(255, 34, 108))
                .timestamp(message.timestamp.to_rfc3339())
                .field(|c| {
                    c.name("Author").value(&format!(
                            "[{a}](https://mods.factorio.com/mods/{a})",
                            a = modification.author
                            ).replace(" ", "%20"))
                })
            .field(|c| {
                c.name("Downloads").value(&format!(
                        "{} downloads",
                        modification.download_count
                        ))
            })
            .field(|c| c.name("Source code").value(&modification.source_path))
                .field(|c| c.name("Homepage").value(&modification.homepage))
                .field(|c| c.name("Last updated").value(&modification.last_updated))
                .field(|c| c.name("Dependencies").value(&modification.dependencies))
                .field(|c| c.name("Tagged").value(&tag_str))
                .field(|c| c.name("Created on").value(&modification.creation_date))
                .field(|c| {
                    c.name("Latest mod version").value(&format!(
                            "{} - [Download]({})",
                            modification.latest_version,
                            modification.download_link
                            ))
                })
            .field(|c| {
                c.name("Factorio version").value(
                    &modification.factorio_version,
                    )
            })
            .field(|c| {
                let popularity = match modification.download_count {
                    0...50 => ":asterisk: New mod! :asterisk:",
                    50...200 => ":seedling: Gathering a following... :seedling:",
                    201...1000 => ":third_place: A decent amount of downloads. :third_place:",
                    1001...2000 => ":second_place: Pretty popular! :second_place:",
                    2001...5000 => ":first_place: Very very popular! :first_place:",
                    5001...10000 => ":heart: Extremely popular! :heart:",
                    100001...105000 => {
                        ":military_medal: Beloved by the community! :military_medal:"
                    }
                    105001...200000 => ":medal: Nearing the Hall of Fame! :medal:",
                    // Anything beyond that
                    _ => ":trophy: Hall of Fame! :trophy:",
                };
                c.name("Popularity").value(popularity)
            })
        })
    });
    if let Err(error) = result {
        println!("Got error with sending embed of mod: {:?}", error);
        return false;
    }
    true
}

/// Creates an embed based on the given modder data. {{{1
fn make_modder_embed(modder: Modder, message: &Message) -> bool {
    message.channel_id.send_message(|a| {
            a.embed(|b| {
                b.title(&format!("Results for {}:", modder.username))
                    .field(|c| c
                           .name("Total downloads:")
                           .value(&modder.total_downloads.to_string()))
                    .field(|c| c
                           .name("Latest mod:")
                           .value(&format!("[{}](https://mods.factorio.com/mods/{}/{})",
                                           modder.latest_mod.title,
                                           modder.username,
                                           modder.latest_mod.name)))
                    .field(|c| c
                           .name("First mod:")
                           .value(&format!("[{}](https://mods.factorio.com/mods/{}/{})",
                                           modder.first_mod.title,
                                           modder.username,
                                           modder.first_mod.name)))
                    .field(|c| c
                           .name("Last updated mod:")
                           .value(&format!("[{}](https://mods.factorio.com/mods/{}/{})",
                                           modder.last_updated_mod.title,
                                           modder.username,
                                           modder.last_updated_mod.name)))
                    .field(|c| c
                           .name("Last updated a mod on:")
                           .value(&modder.last_updated_mod_date))
                    .field(|c| {
                        let popularity = match modder.total_downloads {
                            0...200 => ":asterisk: Learning the ropes! :asterisk:",
                            201...1000 => ":grin: Rising star! :grin:",
                            1001...5000 => ":third_place: Gathering a following... :third_place:",
                            5001...10000 => ":second_place: Excellent mod maker! :second_place:",
                            10001...50000 => ":first_place: Pretty popular! :first_place:",
                            50001...80000 => ":military_medal: Beloved by the community! :military_medal:",
                            80001...100000 => ":medal: Viral! :medal:",
                            100001...500000 => ":100: Hundreds of thousands of players! :100:",
                            500001...1000000 => ":eight_pointed_black_star: Approaching the 1 million club! :eight_pointed_black_star:",
                            1000001...1050000 => ":spaghetti: 1 Million club achieved! :spaghetti:",
                            _ => ":heart_eyes: Superstar modder! :heart_eyes:"
                        };

                        c.name("Popularity").value(popularity)
                    })
                    .color(Colour::from_rgb(255, 34, 108))
                    .timestamp(message.timestamp.to_rfc3339())
            })
        })
    .is_ok()
}

/// Turns a Jsonvalue into a modder. Should be given an array of results. {{{1
fn parse_json_into_modder(username: &str, json: &JsonValue) -> Modder {
    // Turn all entries into Mods
    let mods: Vec<Mod> = json.members().map(|a| parse_json_into_mod(&a)).collect();

    // sum downloads count
    let downloads_vec: Vec<u64> = mods.iter().map(|a| a.download_count).collect();
    let downloads_summed: u64 = downloads_vec.iter().sum();

    // Find out mods of note --------------

    // Get a sorted list of all mod dates
    let mut release_dates: Vec<String> = mods.iter().map(|a| a.creation_date.clone()).collect();
    let mut update_dates: Vec<String> = mods.iter().map(|a| a.last_updated.clone()).collect();
    release_dates.sort_unstable();
    update_dates.sort_unstable();

    // Pair the release date with the mod
    // finding the first release
    let latest_mod = mods.iter().find(|a| {
        a.creation_date == release_dates[release_dates.len()-1]
    }).unwrap().clone();

    // Find first mod
    let first_mod = mods.iter().find(|a| {
        a.creation_date == release_dates[0]
    }).unwrap().clone();

    // Find last updated mod
    let last_updated_mod = mods.iter().find(|a| {
        a.last_updated == update_dates[update_dates.len()-1]
    }).unwrap().clone();

    let last_updated_mod_date = last_updated_mod.last_updated.clone();

    // Return the constructed modder
    Modder {
        username: username.to_owned(),
        total_downloads: downloads_summed,
        latest_mod,
        last_updated_mod_date,
        first_mod,
        last_updated_mod,
    }
}

/// Checks if a user is valid given a username, returns boolean. {{{1
fn is_valid_modder(modder: &str) -> bool {

    let response = reqwest::get(
        format!("https://mods.factorio.com/api/users/{}", modder).as_str(),
        );

    if let Ok(mut response) = response {
        if response.status().is_success() {
            let mut json = String::new();
            let _ = response.read_to_string(&mut json);
            if json.contains("username") {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Turns a jsonValue into a Mod. Assumes this is a direct mod json entry. {{{1
/// This can be provided either by direct link, or by giving it one of the results
/// returned by a mod api search.
fn parse_json_into_mod(json: &JsonValue) -> Mod {
    let (_, downloads, _) = json["downloads_count"].as_number().unwrap().as_parts();
    let thumbnail = if !json["first_media_file"].is_null() {
        format!("{}", json["first_media_file"]["urls"]["thumb"])
    } else {
        String::from("http://i.imgur.com/ckaei9P.png")
    };

    // Formatting for home page/source page
    let homepage = if !json["homepage"].is_null() && json["homepage"] != "" {
        format!("[Home]({})", json["homepage"])
    } else {
        String::from("None")
    };

    let source = if !json["github_path"].is_null() && json["github_path"] != "" {
        format!("[Source](https://github.com/{})", json["github_path"])
    } else {
        String::from("None")
    };

    // Format tag correctly for untagged mods
    let tag = if !json["tags"].is_empty() && !json["tags"].is_null() {
        Some(format!("{}", json["tags"][0]["title"]))
    } else {
        None
    };

    // Fix dates
    let mut date = format!("{}", json["created_at"]);
    if let Some(index) = date.find(" ") {
        date.truncate(index as usize);
    }

    let mut update_date = format!("{}", json["updated_at"]);
    if let Some(index) = update_date.find(" ") {
        update_date.truncate(index as usize);
    }

    let mut deps = String::from("No dependencies.");
    let dependencies_json = &json["latest_release"]["info_json"]["dependencies"];
    if !dependencies_json.is_null() && !dependencies_json.is_empty() {
        deps.clear();
        let mut deps_counter = 0;
        for entry in dependencies_json.members() {
            let entry_str = format!("{}\n", entry);
            // If the dependency is optional, just count it
            if entry_str.starts_with("?") {
                deps_counter += 1;
            } else {
                deps.push_str(&entry_str);
            }
        }
        if deps_counter == 1 {
            deps.push_str("...and 1 optional dependency.");
        } else if deps_counter > 1 {
            deps.push_str(&format!("...and {} optional dependencies.", deps_counter));
        }
    }

    //Download link
    let download_link = format!(
        "https://mods.factorio.com{}",
        json["latest_release"]["download_url"]
        );

    // Make and return the mod
    Mod {
        creation_date: date,
        last_updated: update_date,
        name: format!("{}", json["name"]),
        summary: format!("{}", json["summary"]),
        author: format!("{}", json["owner"]),
        title: format!("{}", json["title"]),
        latest_version: format!("{}", json["latest_release"]["version"]),
        factorio_version: format!("{}", json["latest_release"]["factorio_version"]),
        tag,
        thumb: thumbnail,
        download_count: downloads,
        download_link,
        homepage,
        dependencies: deps,
        source_path: source,
        link: format!(
            "https://mods.factorio.com/mods/{}/{}",
            json["owner"],
            json["name"]
            ).replace(" ", "%20"),
    }
}


/// Makes a request, either returning empty, or the successful mod json {{{1
fn make_request(request: &String) -> JsonValue {
    let response = reqwest::get(
        format!("https://mods.factorio.com/api/mods?q={}", request).as_str(),
        );

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
    message
        .channel_id
        .send_message(|a| {
            a.embed(|b| {
                b.title("Search results:")
                    .description(&serialize_search_results(&results))
                    .color(Colour::from_rgb(255, 34, 108))
                    .timestamp(message.timestamp.to_rfc3339())
            })
        })
    .is_ok()
}

/// Takes a json array as input, returns a string of the search results serialized. {{{1
fn serialize_search_results(results: &JsonValue) -> String {
    let mut final_string = String::new();
    let mut outdated_counter = 0;

    for entry in results.members().take(10) {
        let version = format!("{}", entry["latest_release"]["factorio_version"]);
        if version.len() < 4 {
            continue; //Version is not complete
        }
        let version: f32 = version[..4].parse::<f32>().unwrap(); //only parse the first 4 chars, so x.xx
        if version < 0.15 {
            outdated_counter += 1;
            continue; //Filter mods that aren't for current stable or higher
        }

        let mut encoded_name = format!("{}", entry["name"]);
        // URLS can't have spaces
        encoded_name = encoded_name.replace(" ", "%20");
        final_string += format!(
            "- [{}](https://mods.factorio.com/mods/{}/{}) by {}\n",
            entry["title"],
            entry["owner"],
            encoded_name,
            entry["owner"]
            ).as_str();
    }
    if final_string.is_empty() {
        format!(
            "No results for current factorio stable version or higher.
                \n{} outdated mods were found.",
                outdated_counter
               )
    } else {
        format!(
            "{}\n...and {} outdated mod(s).",
            final_string,
            outdated_counter
            )
    }
}

/// Links a mod into chat based on args. {{{1
command!(linkmod(_context, message) {
    let mut request = String::new();
    let server_prefix = get_prefix_for_guild(&message.guild_id().unwrap());

    if message.content_safe().starts_with(format!("{}linkmod", server_prefix).as_str()) {
        request = fix_message(message.content_safe(), "linkmod");
    } else if message.content_safe().starts_with(format!("{}mod", server_prefix).as_str()) {
        request = fix_message(message.content_safe(), "mod");
    }

    let _ = message.channel_id.broadcast_typing();

    // Check arg validity
    if request.is_empty() {
        if let Err(_) = send_error_embed(&message, "Expected a mod to search for.") {
            say_into_chat(&message, "Expected a mod to search for.");
        }
        return Err(String::from("User didn't provide an argument."));
    }

    // Make the mod api request
    let returned = make_request(&request);
    if !returned.is_empty() {
        let returned_results = &returned["results"];

        if !returned_results.is_null() && !returned_results.is_empty() {
            // If there's only one search result
            if returned_results.len() == 1 {
                let modification = parse_json_into_mod(&returned_results[0]);
                if !make_mod_embed(modification, &message) {
                    say_into_chat(&message, "Unable to make an embed here.");
                    return Err(String::from("Couldn't make an embed."));
                } else {
                    return Ok(());
                }
            }
            // More than one search result, so let's list them
            for entry in returned_results.members() {
                let modification = parse_json_into_mod(&entry);

                // Check if the match is close enough, both on the
                // internal name and the title
                if levenshtein(&modification.name, &request) <= DISTANCE_SENSITIVITY
                    || levenshtein(&modification.title, &request) <= DISTANCE_SENSITIVITY {
                        // Got a match on this entry, so let's send it
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
            if let Err(_) = send_error_embed(&message, "There are no results for that query.") {
                say_into_chat(&message, "There are no results for that query.");
            }
            return Err(String::from("Didn't find any results for the request."));
        }
    } else {
        if let Err(_) = send_error_embed(&message, "Couldn't get a response back from the mod portal.") {
            say_into_chat(&message, "Couldn't get a response back from the mod portal.");
        }
        return Err(String::from("Failed to get a response back from the mod portal."));
    }
});

/// When given a username, returns an embed of information about {{{1
/// a modder, such as number of mods, most recent mod, total downloads, etc.
command!(modder(_context, message) {
    let modder_username = fix_message(message.content_safe(), "modder");
    let _ = message.channel_id().broadcast_typing();

    // Check if the username provided is valid
    if !modder_username.is_empty() && is_valid_modder(&modder_username) {
        let results = make_request(&modder_username);
        if !results.is_empty() {
            let modder = parse_json_into_modder(&modder_username, &results["results"]);
            if !make_modder_embed(modder, &message) {
                say_into_chat(&message, "Couldn't make an embed here.");
            } else {
                return Ok(());
            }
        } else {
            if let Err(_) = send_error_embed(&message, "Couldn't find any results. The mod portal might be down.") {
                say_into_chat(&message, "Couldn't find any results. The mod portal might be down.");
            }
            return Err(String::from("Couldn't reach the mod portal."));
        }
    } else {
        if let Err(_) = send_error_embed(&message, "Sorry, no modder by that username was found.") {
            say_into_chat(&message, "Sorry, no modder by that username was found.");
        }
            return Err(String::from("Invalid username."));
    }
});

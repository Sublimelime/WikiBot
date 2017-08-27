extern crate serenity;
extern crate rand;

use self::rand::Rng;
use common_funcs::*;
use std::process::Command;
use std::process;
use self::serenity::utils::Colour;
use constants::*;

lazy_static! {
    static ref REPLIES: Vec<&'static str> = include_str!("replies.txt").lines().collect();
}

/// Your standard ping command, replies with random replies {{{1
command!(ping(_context, message) {
    let reply = rand::thread_rng().choose(&REPLIES).unwrap();
    reply_into_chat(&message, reply);
});

/// Stops the bot, shutting down the process. {{{1
command!(stop_process(_context, message) {
    reply_into_chat(&message, "Stopping bot. Goodbye.");
    process::exit(0);
});

/// Prints various info about the bot itself. {{{1
command!(info(_context, message) {
    let mut server_prefix = get_prefix_for_guild(&message.guild_id().unwrap());
    if server_prefix.is_empty() {
        server_prefix = String::from("None, use mention.");
    }

    let mut reply = String::from("A simple bot that fetches information related to factorio.\nFor help and a list of commands, use ");
    reply.push_str(&server_prefix);
    reply.push_str("help. All commands that do not take arguments support talking after the commands with ||, commands that take arguments support it if it says so in help.
                   \nThanks, and enjoy! For info about the host of this bot, run the `host` command.");
    let _ = message.channel_id.send_message(|a| a
                                            .embed(|b| b
                                                   .title("WikiBot")
                                                   .description(reply.as_str())
                                                   .field(|c| c.name("Author").value("Gangsir"))
                                                   .field(|c| c.name("Current maintainer").value("Gangsir"))
                                                   .field(|c| c.name("Contact").value("I'm in the factorio discord server,
                                                                                        or you can reddit PM me [here](https://www.reddit.com/message/compose?to=Gangsir)."))
                                                   .field(|c| c.name("Programming language").value("Made in [Rust](https://www.rust-lang.org)."))
                                                   .field(|c| c.name("Library").value("Made with [Serenity](https://crates.io/crates/serenity)."))
                                                   .field(|c| c.name("Source code").value("[Source](https://github.com/TheRealGangsir/WikiBot)"))
                                                   .field(|c| c.name("Local prefix").value(&server_prefix))
                                                   .timestamp(message.timestamp.to_rfc3339())
                                                   .color(Colour::from_rgb(255, 255, 255))
                                                   ));
});

/// Prints current system status, including uptime {{{1
/// of the bot into chat.
command!(uptime(_context, message) {
    // Run commands and check results
    if let Ok(uptime_result) = Command::new("uptime").output() {
        if let Ok(mem_result) = Command::new("free").arg("-ht").output() {
            let uptime_result = String::from_utf8(uptime_result.stdout).unwrap();
            let mem_result = String::from_utf8(mem_result.stdout).unwrap();

            // Make embed reply
            let _ = message.channel_id.send_message(|a| a
                                                    .embed(|b| b
                                                           .title("Status")
                                                           .description("Current bot status:")
                                                           .field(|c| c.name("Uptime").value(format!("```{}```", uptime_result).as_str()))
                                                           .field(|c| c.name("Memory").value(format!("```{}```", mem_result).as_str()))
                                                           .timestamp(message.timestamp.to_rfc3339())
                                                           .color(Colour::from_rgb(255, 255, 255))
                                                          ));
        } else {
            say_into_chat(&message, "Sorry, an error occured on my host.");
            return Err(String::from("Unable to run free command."));
        }
    } else {
        say_into_chat(&message, "Sorry, an error occured on my host.");
        return Err(String::from("Unable to run uptime command."));
    }
});

/// Prints data about the host into the chat. {{{1
command!(host(_context, message) {
    use std::env::consts::*;
    let mut uptime;

    if let Ok(result) = Command::new("uptime").arg("-p").output() {
        uptime = String::from_utf8(result.stdout).unwrap();
    } else {
        return Err(String::from("Unable to run uptime command."));
    }
    let embed_result = message.channel_id.send_message(|a| a
                                                       .embed(|b| b
                                                              .description("Powered by [Rust Stable](https://www.rust-lang.org).")
                                                              .field(|c| c.name("Host OS:").value(OS))
                                                              .field(|c| c.name("Host processor arch:").value(ARCH))
                                                              .field(|c| c.name("Uptime:").value(uptime.as_str()))
                                                              .timestamp(message.timestamp.to_rfc3339())
                                                              .color(Colour::from_rgb(255,255,255))
                                                             ));

    if let Err(_) = embed_result {
        reply_into_chat(&message, format!("Host OS: {}\nHost Arch: {}\nCurrent uptime: {}
                                  \n Powered by Rust(stable). https://www.rust-lang.org",
                                  OS,
                                  ARCH,
                                  uptime
                                  ));
    }
});

/// Links a page on the wiki. {{{1
command!(page(_context, message) {
    let mut final_message = String::from("https://wiki.factorio.com/");
    let server_prefix = get_prefix_for_guild(&message.guild_id().unwrap());

    // Remove command from message content, and code-ify it
    let mut modified_content = String::new();
    if message.content_safe().starts_with(format!("{}page", server_prefix).as_str()) {
        modified_content = fix_message(message.content_safe(), "page ", &server_prefix);
    } else if message.content_safe().starts_with(format!("{}link", server_prefix).as_str()) {
        modified_content = fix_message(message.content_safe(), "link ", &server_prefix);
    }

    modified_content = modified_content.replace(" ", "_");

    final_message.push_str(modified_content.as_str()); //add the specified page to the end

    // Post link back into chat
    say_into_chat(&message, final_message);
});

/// Creates a link to an older FFF. {{{1
command!(fff_old(_context, message) {
    let mut final_message = String::from("https://factorio.com/blog/post/fff-");
    let server_prefix = get_prefix_for_guild(&message.guild_id().unwrap());

    let mut modified_content = String::new();

    if message.content_safe().starts_with(format!("{}fff-old", server_prefix).as_str()) {
        modified_content = fix_message(message.content_safe(), "fff-old ", &server_prefix);
    } else if message.content_safe().starts_with(format!("{}blog-old", server_prefix).as_str()) {
        modified_content = fix_message(message.content_safe(), "blog-old ", &server_prefix);
    }

    final_message.push_str(modified_content.as_str());
    reply_into_chat(&message, final_message);
});

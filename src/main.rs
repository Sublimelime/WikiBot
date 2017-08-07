#[macro_use]
extern crate serenity;
extern crate inflector;
extern crate rand;

use std::fs::File;
use rand::Rng;
use std::io::prelude::*;
use inflector::Inflector;
use serenity::client::Client;
use serenity::model::UserId;
use serenity::framework::help_commands;

// Global config vars
static PREFIX: &'static str = "+";

fn main() {
    let filename = "token.txt";
    let mut file = File::open(filename).expect("Token file not found");

    let mut token = String::new();
    file.read_to_string(&mut token).expect(
        "Something went wrong reading the token file",
    );

    let token = token.trim(); //Remove the newline from the end of the string if present

    // Login with a bot token from the environment
    let mut client = Client::new(&token[..]);

    client.with_framework(|f| {
        f.configure(|c|
                    c.prefix(PREFIX) // set the bot's prefix to prefix declared as global
                    .ignore_bots(true) //Ignore other bots
                    .ignore_webhooks(true)
                    .on_mention(true) // Allow mentioning the bot to use commands
                    .owners(vec![UserId(152193207726243840)].into_iter().collect()) //setup author to be owner

                    )
            .on("ping", ping)
            .on("info", info)
            .command("help", |c| c.exec_help(help_commands::with_embeds))
            .on("page", page)
    });

    println!("Bot configured, with prefix {}, now running...", PREFIX);

    let _ = client.start(); //Start bot
}

command!(ping(_context, message) {
    let replies = ["Pong!", "Marco!", "Roger!", "I hear ya!",
    "Good day.", "Hello!", "What's up?","I'm alive!",
    "Hearing you loud and clear.","PiNg!", "Yep, still here.",
    "Nah, totally offline.", "Sup?", "What's a bot gotta do to get some sleep around here?",
    "Running.", "Uptime is my prime focus."];
    let random = rand::thread_rng().gen_range(0, replies.len());

    let _ = message.reply(replies[random]);
});

command!(info(_context, message) {
    let mut reply = String::from("Author: Gangsir\nDesc: A simple bot that fetches information related to factorio.\nFor help, use ");
    reply.push_str(PREFIX);
    reply.push_str("help. Thanks, and enjoy!");
    let _ = message.reply(&reply[..]);
});


command!(page(_context, message) {
    let mut final_message = String::from("https://wiki.factorio.com/");
    let modified_content = message.content_safe().replace("page ", "").replace(PREFIX,"");

    // Remove command from message content, and code-ify it
    let modified_content = modified_content.clone().to_sentence_case().replace(" ", "_");

    final_message.push_str(&modified_content[..]); //add the specified page to the end

    // Post link back into chat
    if let Err(error) = message.channel_id.say(&final_message[..]) {
        println!("[Error] Unable to send reply message for page command. Error is {}", error);
    }

});

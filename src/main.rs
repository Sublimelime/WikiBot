#[macro_use]
extern crate serenity;

use std::fs::File;
use std::io::prelude::*;
use serenity::client::Client;
use serenity::framework::Framework;
use serenity::model::UserId;
use serenity::framework::help_commands;

mod commands;
use commands::*;

// Global config vars
pub const PREFIX: &'static str = "+";

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
            // META GROUP -------------------------
            .group("Meta", |g| g
                   .command("ping", |c|
                            c.desc("Replies to the ping with a message. Used to check if the bot is working.")
                            .max_args(0)
                            .exec(ping))
                   .command("info", |c|
                            c.desc("Prints out info about the bot.")
                            .batch_known_as(vec!["about", "what", "?"])
                            .exec(info))
                   .command("help", |c| c.exec_help(help_commands::with_embeds)))

            // WIKI GROUP -------------------------
            .group("Wiki", |g| g
                   .command("page", |c|
                            c.desc("Takes a page name, and prints out a link to the wiki of that page.")
                            .min_args(1)
                            .known_as("link")
                            .usage("\nPass it a page name, such as iron plate. It'll return a link.
                            \nIf you wish to keep talking after this command, use two pipes \"||\" to end the command and begin your chat.")
                            .example("iron plate")
                            .help_available(true)
                            .exec(page))

                   .command("ratios", |c|
                            c.desc("Returns a list of all registered ratios.")
                            .max_args(0)
                            .help_available(true)
                            .exec(ratios))
                   .command("modapi", |c|
                            c.desc("Takes a link, and creates an embed detailing the linked field.")
                            .min_args(1)
                            .help_available(true)
                            .exec(modapi))
                  )
            // RATIOS GROUP --------------------------
            .group("Ratios", |g| g
                   .prefix("ratio")
                   .command("add", |c| c
                            .desc("Adds a ratio to the list of created ratios.")
                            .min_args(2)
                            .example("name 1:2:3")
                            .usage("name ratio\nProvide a name for the ratio, and the ratio itself.")
                            .exec(ratio_add))
                   .command("get", |c| c
                            .desc("Retrieves a ratio and prints it into chat.")
                            .min_args(1)
                            .example("name")
                            .usage("name\nProvide a name for the ratio to get.")
                            .exec(ratio_get))
                  )

    });

    // Init
    println!("Bot configured with prefix {}, now running...", PREFIX);

    let _ = client.start(); //Start bot
}

#[macro_use]
extern crate serenity;

use std::fs::File;
use std::io::prelude::*;
use serenity::client::Client;
use serenity::framework::Framework;
use serenity::model::{UserId, Permissions};
use serenity::framework::{help_commands, DispatchError};

mod commands;
mod ratios;
use commands::*;
use ratios::*;

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

    // Create defined perms for what is a person of power
    let mut is_powerful_perms: Permissions = Permissions::empty();
    // Add managing roles to definition
    is_powerful_perms.insert(Permissions::from_bits_truncate(0x10000000));

    client.with_framework(move |f| {
        f.configure(|c|
                    c.prefix(PREFIX) // set the bot's prefix to prefix declared as global
                    .ignore_bots(true) //Ignore other bots
                    .ignore_webhooks(true)
                    .allow_dm(true)
                    .allow_whitespace(true)
                    .on_mention(true) // Allow mentioning the bot to use commands
                    .owners(vec![UserId(152193207726243840)].into_iter().collect()) //setup author to be owner

                   )
            // META GROUP -------------------------
            .group("Meta", |g| g
                   .command("ping", |c|
                            c.desc("Replies to the ping with a message. Used to check if the bot is working.")
                            .exec(ping))
                   .command("info", |c|
                            c.desc("Prints out info about the bot.")
                            .batch_known_as(vec!["about", "what", "?"])
                            .exec(info))
                   .command("help", |c| c.exec_help(help_commands::with_embeds)))

            // WIKI GROUP -------------------------
            .group("Wiki", |g| g
                   .command("page", |c|
                            c.desc("Takes a page name, and prints out a link to the wiki of that page.
                                   \nIf you wish to keep talking after this command, use two pipes \"||\" to end the command and begin your chat.")
                            .known_as("link")
                            .example("iron plate")
                            .help_available(true)
                            .exec(page))

                   .command("ratios", |c|
                            c.desc("Returns a list of all registered ratios.")
                            .help_available(true)
                            .exec(ratios))
                   .command("modapi", |c|
                            c.desc("Takes a link, and creates an embed detailing the linked field.")
                            .help_available(true)
                            .exec(modapi))
                  )
            // RATIOS GROUP --------------------------
            .group("Ratios", |g| g
                   .prefix("ratio")
                   .command("add", |c| c
                            .desc("Adds a ratio to the list of created ratios.
                                  \nProvide a name for the ratio, and the ratio itself. Quotes are required around each arg.")
                            .min_args(2)
                            .use_quotes(true)
                            .max_args(2)
                            .required_permissions(is_powerful_perms)
                            .example("\"name\" \"1:2:3\"")
                            .usage("\"name\" \"ratio\"")
                            .exec(ratio_add))

                   .command("get", |c| c
                            .desc("Retrieves a ratio and prints it into chat.
                                  \nProvide a name for the ratio to get.
                                  \nIf you wish to keep talking after this command, use two pipes \"||\" to end the command and begin your chat.")
                            .min_args(1)
                            .example("steam")
                            .usage("name")
                            .exec(ratio_get))
                  )
            .on_dispatch_error(|_ctx, msg, error| {
                if let DispatchError::RateLimited(seconds) = error {
                    say_into_chat(&msg, &format!("Try this again in {} seconds.", seconds));
                }
            })
        .before(|_, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);
            true
        })
    });

    client.on_ready(|_ctx, ready| {
        println!("{} is connected!", ready.user.name);
    });

    // Init
    println!("Bot configured with prefix {}, now running...", PREFIX);

    let _ = client.start(); //Start bot
}

#[macro_use]
extern crate serenity;

use std::fs::File;
use std::io::prelude::*;
use serenity::client::Client;
use serenity::model::{UserId, Permissions};
use serenity::framework::{help_commands, DispatchError};

mod commands;
mod ratios;
mod levenshtein;
mod web_requesting;
use commands::*;
use ratios::*;
use web_requesting::*;

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
        f.simple_bucket("slowly", 5)
            .simple_bucket("super-slowly", 10)
            .simple_bucket("occasionally", 30)

            .configure(|c|
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
                   .command("uptime", |c| c
                            .desc("Prints out info about the bot's uptime, and system status.")
                            .known_as("status")
                            .bucket("slowly")
                            .exec(uptime))
                   .command("host", |c| c
                            .desc("Prints out info about the host's uptime, and system info.")
                            .bucket("super-slowly")
                            .exec(host))
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
                            .bucket("super-slowly")
                            .exec(ratios))
                   .command("fff", |c| c
                            .desc("Returns a link to the newest FFF. Due to expensive operations, can only be used once every 30 seconds.")
                            .help_available(true)
                            .known_as("blog")
                            .bucket("occasionally")
                            .exec(fff))
                   .command("version", |c| c
                            .desc("Returns the number of the latest version for stable and experimental. Due to expensive operations, can only be used once every 30 seconds.")
                            .help_available(true)
                            .bucket("occasionally")
                            .exec(version))
                   .command("fff-old", |c| c
                            .desc("Returns a link to an older FFF. Provide it with the number of the FFF in question.
                                  \nThis command supports typing after the command, simply end it with two pipes, ||")
                            .help_available(true)
                            .example("24")
                            .min_args(1)
                            .known_as("blog-old")
                            .exec(fff_old))
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
                            .help_available(true)
                            .usage("\"name\" \"ratio\"")
                            .exec(ratio_add))

                   .command("get", |c| c
                            .desc("Retrieves a ratio and prints it into chat.
                                  \nProvide a name for the ratio to get.
                                  \nIf you wish to keep talking after this command, use two pipes \"||\" to end the command and begin your chat.")
                            .min_args(1)
                            .example("steam")
                            .usage("name")
                            .help_available(true)
                            .exec(ratio_get))
                   .command("delete", |c| c
                            .desc("Deletes a ratio. Can only be used by moderators.")
                            .min_args(1)
                            .example("steam")
                            .help_available(true)
                            .required_permissions(is_powerful_perms)
                            .usage("name")
                            .exec(ratio_delete))
                   .command("deleteall", |c| c
                            .desc("Deletes all ratios. Can only be used by moderators.")
                            .known_as("clear")
                            .required_permissions(is_powerful_perms)
                            .help_available(true)
                            .exec(ratio_deleteall))
                   .command("set", |c| c
                            .desc("Sets an existant ratio to a different value. Can only be used by moderators.")
                            .min_args(2)
                            .max_args(2)
                            .help_available(true)
                            .use_quotes(true)
                            .example("\"steam\" \"1:2:3\"")
                            .required_permissions(is_powerful_perms)
                            .usage("\"name\" \"new value\"")
                            .exec(ratio_set))
                   )
                   .on_dispatch_error(|_ctx, msg, error| {
                       match error {
                           DispatchError::RateLimited(seconds) => {
                                println!("Bot is being rate limited, or user triggered bucket.");
                                say_into_chat(&msg, format!("Slow down there partner! Try this command again in {} seconds.", seconds));
                           }
                           DispatchError::LackOfPermissions(_) | DispatchError::OnlyForOwners => {
                               let _ = send_error_embed(&msg, "Sorry, you don't have permission to do that.");
                           }
                           DispatchError::NotEnoughArguments{min, given} => {
                               let _ = send_error_embed(&msg, format!("I'm sorry, input was incomplete, I was expecting {} args, but you sent {}.", min, given).as_str());
                           }
                           DispatchError::TooManyArguments{max, given} => {
                               //Do nothing, this happens
                           }
                           _ => println!("Got unknown dispatch error.")
                       }
                   })
        .before(|_, msg, command_name| {
            // Print info about the command use into log
            println!("Got command '{}' by user '{}#{}'",
                     command_name,
                     msg.author.name,
                     msg.author.discriminator);
            true
        })
    });

    client.on_ready(|ctx, ready| {
        println!("{} is connected!", ready.user.name);
        ctx.set_game_name(format!("{}help for help!", PREFIX).as_str());
    });

    // Init
    println!(
        "Bot configured with prefix {}, now waiting for connection...",
        PREFIX
    );

    let _ = client.start(); //Start bot
}

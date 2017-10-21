extern crate serenity;
#[macro_use]
extern crate wikibot;

use serenity::prelude::*;
use serenity::model::*;
use serenity::model::event::*;
use serenity::framework::standard::*;

use std::fs::File;
use std::io::prelude::*;

use wikibot::commands::*;
use wikibot::common_funcs::*;
use wikibot::constants::{self, install_prefixes};

//Eventhandler setup {{{1
struct Handler;

impl EventHandler for Handler {
    // Ready/Resume handlers {{{2
    fn on_ready(&self, ctx: Context, ready: Ready) {
        log_status!("{} is connected!", ready.user.name);
        ctx.set_game_name(format!("@{} help for help!", constants::BOT_NAME).as_str());
        // List current servers into log
        println!("{} is now online in the following guilds:", ready.user.name);
        for entry in ready.guilds.iter() {
            println!("ID: {}", entry.id());
        }
    }

    fn on_resume(&self, ctx: Context, _res: ResumedEvent) {
        log_status!("Resumed after a disconnect.");
        ctx.set_game_name(format!("@{} help for help!", constants::BOT_NAME).as_str());
    }
}

/// Main function. {{{1
fn main() {
    let filename = "token.txt";
    let mut file = File::open(filename).expect("Token file not found.");

    let mut token = String::new();
    file.read_to_string(&mut token).expect(
        "Something went wrong reading the token file",
    );

    let token = token.trim(); //Remove the newline from the end of the string if present

    // Login with a bot token from the environment
    let mut client = Client::new(&token[..], Handler);
    log_init!("Created client with token successfully.");

    // Create defined perms for what is a person of power
    let mut is_powerful_perms: Permissions = Permissions::empty();
    // Add managing roles to definition
    is_powerful_perms.insert(Permissions::from_bits_truncate(0x10000000));

    install_prefixes();
    log_init!("Configured prefixes from file.");

    // Configure client with framework {{{2
    client.with_framework(StandardFramework::new()
            .simple_bucket("slowly", 5)
            .simple_bucket("super-slowly", 10)
            .simple_bucket("occasionally", 20)

            .configure(|c|
                       c.dynamic_prefix(|_ctx, message| {
                           //Get guild id of the current message
                           if let Some(guild_id) = message.guild_id() {
                               let prefixes = constants::PREFIXES.lock().unwrap();
                               return prefixes.get(&guild_id).cloned();
                           } else {
                               log_info!("Guild ID was not in the cache.");
                               None
                           }
                       })
                       .ignore_bots(true) //Ignore other bots
                       .ignore_webhooks(true)
                       .case_insensitivity(true)
                       .allow_dm(true)
                       .allow_whitespace(true)
                       .on_mention(true) // Allow mentioning the bot to use commands
                       .owners(vec![UserId(152193207726243840)].into_iter().collect())) //setup author to be owner
            // META GROUP ------------------------- {{{3
            .group("Meta", |g| g
                   .command("ping", |c|
                            c.desc("Replies to the ping with a message. Used to check if the bot is working.")
                            .exec(ping))
                   .command("info", |c|
                            c.desc("Prints out info about the bot.")
                            .batch_known_as(vec!["about", "what", "?"])
                            .exec(info))
                   .command("whois", |c|
                            c.desc("Prints out info about a user in a server.
                                   Defaults to the summoner when no user is provided.")
                            .usage("@user#number")
                            .example("@Gangsir#2512")
                            .help_available(false)
                            .guild_only(true)
                            .exec(whois))
                   .command("uptime", |c| c
                            .desc("Prints out info about the bot's uptime, and system status. Can only be used by the owner.")
                            .known_as("status")
                            .help_available(false)
                            .owners_only(true)
                            .exec(uptime))
                   .command("stop", |c| c
                            .desc("Stops the bot, shutting it down. Can only be used by the owner.")
                            .known_as("exit")
                            .help_available(false)
                            .owners_only(true)
                            .exec(stop_process))
                   .command("host", |c| c
                            .desc("Prints out info about the host's uptime, and system info.")
                            .bucket("super-slowly")
                            .exec(host))
                   .command("help", |c| c
                            .exec_help(help_commands::with_embeds))
                   )
                   // CONFIG GROUP ----------------------- {{{3
                   .group("Config", |g| g
                          .command("prefix", |c| c
                                   .desc("Registers/Changes a prefix on this server.
                                         \nOnly one prefix per server can be set, ensure it doesn't collide with any other bots.")
                                   .min_args(1)
                                   .required_permissions(is_powerful_perms)
                                   .guild_only(true)
                                   .example("+")
                                   .help_available(true)
                                   .exec(register_prefix))
                         )
                   // WIKI GROUP ------------------------- {{{3
                   .group("Wiki", |g| g
                          .command("page", |c|
                                   c.desc("Takes a page name, and prints out a link to the wiki of that page.
                                   \nIf you wish to keep talking after this command, use two pipes \"||\" to end the command and begin your chat.")
                                   .batch_known_as(vec!["link", "wiki"])
                                   .example("iron plate")
                                   .help_available(true)
                                   .exec(page))
                          .command("api", |c|
                                   c.desc("Takes a search string, returns a search of the modding api for that string.
                                          \nIf you wish to keep talking after this command, use two pipes \"||\" to end the command and begin your chat.")
                                   .example("player")
                                   .help_available(true)
                                   .exec(search_api))
                          .command("fff", |c| c
                                   .desc("Returns a link to the newest FFF. Due to expensive operations, can only be used once every 30 seconds.")
                                   .help_available(true)
                                   .known_as("blog")
                                   .bucket("occasionally")
                                   .exec(fff))
                          .command("linkmod", |c| c
                                   .desc("When provided with the name of a mod, it will return an embed of all the data on that mod.
                                         \nWhen provided with anything else, preforms a search.
                                         \nFails if that search doesn't return any results.
                                         \nThe embed also provides popularity evaluation based on the download counts.
                                         \nThis command supports pipe syntax.")
                                   .help_available(true)
                                   .example("Achiever")
                                   .known_as("mod")
                                   .bucket("slowly")
                                   .exec(linkmod))
                          .command("modder", |c| c
                                   .desc("When provided with the name of a modder, it will return an embed of all the data on that modder.
                                         \nFails if the username is invalid, has not made any mods, or the api cannot be reached.
                                         \nNote that the name of the modder must be spelled and capitalized correctly.
                                         \nThe embed also provides popularity evaluation based on the download counts.
                                         \nThis command also supports pipe syntax.")
                                   .help_available(true)
                                   .example("Gangsir")
                                   .bucket("super-slowly")
                                   .exec(modder))
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
                          .command("recipe", |c| c
                                   .desc("Returns a recipe of an item, machine, or process.
                                         \nIf the name of the object isn't internal, it will be autocorrected to the closest match.
                                         \nThis command uses fancy algorithms to correct names and run, and so can only be used every few seconds.
                                         \nThis command also supports typing after the command, end the command with two pipes, ||.")
                                   .help_available(true)
                                   .bucket("super-slowly")
                                   .usage("<some item, process....>")
                                   .example("beacon")
                                   .exec(recipe))
                          )
                          // FAQ GROUP -------------------------- {{{3
                          .group("FAQ System", |g| g
                                 .command("faq-list", |c|
                                          c.desc("Returns a list of all registered faqs.")
                                          .help_available(true)
                                          .guild_only(true)
                                          .bucket("super-slowly")
                                          .exec(faqs))
                                 .command("faq-add", |c| c
                                          .desc("Adds a ratio to the list of created ratios.
                                  \nProvide a name for the ratio, and the ratio itself.
                                  \nThe name of the ratio can only be one word, use underscores if necessary.
                                  \n Can only be used by moderators.")
                                          .required_permissions(is_powerful_perms)
                                          .example("steam This is the ratio for steam:.....")
                                          .guild_only(true)
                                          .min_args(2)
                                          .max_args(2)
                                          .help_available(true)
                                          .exec(faq_add))
                                 .command("faq", |c| c
                                          .desc("Retrieves a faq and prints it into chat.
                                  \nProvide a name for the faq to get.
                                  \nIf you wish to keep talking after this command, use two pipes \"||\" to end the command and begin your chat.")
                                          .example("steam")
                                          .known_as("faw")
                                          .guild_only(true)
                                          .usage("name")
                                          .help_available(true)
                                          .exec(faq_get))
                                 .command("faq-delete", |c| c
                                          .desc("Deletes a faq. Can only be used by moderators.")
                                          .min_args(1)
                                          .example("steam")
                                          .help_available(true)
                                          .guild_only(true)
                                          .required_permissions(is_powerful_perms)
                                          .usage("name")
                                          .exec(faq_delete))
                                 .command("faq-deleteall", |c| c
                                          .desc("Deletes all faq. Can only be used by moderators.")
                                          .required_permissions(is_powerful_perms)
                                          .help_available(true)
                                          .guild_only(true)
                                          .exec(faq_deleteall))
                                 .command("faq-set", |c| c
                                          .desc("Sets an existant faq to a different value. Can only be used by moderators.")
                                          .help_available(true)
                                          .example("steam This is the new ratio for steam:....")
                                          .guild_only(true)
                                          .num_args(2)
                                          .required_permissions(is_powerful_perms)
                                          .exec(faq_set))
                                 )
                                 // DISPATCH ERRORS ----------------------------- {{{3
                                 .on_dispatch_error(|_ctx, msg, error| {
                                     match error {
                                         DispatchError::RateLimited(seconds) => {
                                             log_info!("User triggered ratelimit bucket.");
                                             say_into_chat(&msg, format!("Woah! This command is hard for me to do,
                                                                         could you try again in {} seconds? :sweat_smile:", seconds));
                                         }
                                         DispatchError::CommandDisabled(_) => {
                                             say_into_chat(&msg, "Sorry, this command is disabled due to abuse or unstability.");
                                         }
                                         DispatchError::BlockedUser | DispatchError::BlockedGuild => {
                                             log_info!("Ignoring command by blocked user/guild...");
                                         }
                                         DispatchError::LackOfPermissions(_) | DispatchError::OnlyForOwners | DispatchError::CheckFailed(_) => {
                                             send_error_embed_or_say(&msg, "Sorry, you don't have permission to do that.");
                                         }
                                         DispatchError::OnlyForDM | DispatchError::OnlyForGuilds => {
                                             send_error_embed_or_say(&msg, "Sorry, I can't preform this command here.
                                                                      It can only be used in either guilds or DMs.");
                                         }
                                         DispatchError::NotEnoughArguments{min, given} => {
                                             send_error_embed_or_say(&msg, format!("I'm sorry, input was incomplete, I was expecting {} args,
                                                                                   but you sent {}.", min, given).as_str());
                                         }
                                         _ => log_error!("Got unknown dispatch error.")
                                     };
                                 })
        // BEFORE/AFTER {{{3
        .before(|_, msg, command_name| {
            // Print info about the command use into log
            log_info!(
                "Got command '{}' by user '{}#{}', running...",
                command_name,
                msg.author.name,
                msg.author.discriminator);
            true
        })
        .after(|_, msg, command_name, error| {
            // Print info about the command use into log
            if let Err(reason) = error {
                make_log_entry!("CMD-Error",
                                "Got error during command '{}' by user '{}#{}', error is: {:?}",
                                command_name,
                                msg.author.name,
                                msg.author.discriminator,
                                reason);
            } else {
                log_info!(
                    "Completed command '{}' by user '{}#{}'",
                    command_name,
                    msg.author.name,
                    msg.author.discriminator);
            }
        }));

    log_init!("Configured bot succesfully.");
    // Init {{{2
    log_status!("Now waiting for connection...");

    if let Err(why) = client.start() {
        println!("Err with client: {:?}", why);
    } else {
        println!("Client exited succesfully.");
    }
}

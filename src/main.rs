#[macro_use]
extern crate serenity;

use std::fs::File;
use std::io::prelude::*;
use serenity::client::Client;

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
        f.configure(|c| c.prefix(PREFIX)) // set the bot's prefix to prefix declared as global
            .on("ping", ping)
    });

    println!("Bot configured, with prefix {}, now running...", PREFIX);

    let _ = client.start();
}

command!(ping(_context, message) {
    let _ = message.reply("Pong!");
});

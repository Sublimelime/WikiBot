#[macro_use] extern crate serenity;

use serenity::client::Client;

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new("MzQxNjE3NTk1MjU2MTQzODcy.DGDyAQ.zCRbH5wUv-kHOzfSGdvBFGhaRYg");

    client.with_framework(|f| f
                          .configure(|c| c.prefix("+")) // set the bot's prefix to "+"
                          .on("ping", ping));

    client.on_message(|_context, message| {
        println!("{}", message.content);
    });

    let _ = client.start();
}

command!(ping(_context, message) {
    let _ = message.reply("Pong!");
});

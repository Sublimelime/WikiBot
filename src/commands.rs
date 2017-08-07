extern crate inflector;
extern crate rand;

use self::inflector::Inflector;
use self::rand::Rng;

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
    reply.push_str(::PREFIX);
    reply.push_str("help. Thanks, and enjoy!");
    let _ = message.reply(&reply[..]);
});


command!(page(_context, message) {
    let mut final_message = String::from("https://wiki.factorio.com/");

    // Remove command from message
    let mut modified_content = message.content_safe().replace("page ", "").replace(::PREFIX,"");

    // Truncate message to ||
    if let Some(index) = modified_content.find(" ||") {
        modified_content.truncate(index as usize);
    }

    // Remove command from message content, and code-ify it
    modified_content = modified_content.replace(" ", "_");

    final_message.push_str(&modified_content[..]); //add the specified page to the end

    // Post link back into chat
    if let Err(error) = message.channel_id.say(&final_message[..]) {
        println!("[Error] Unable to send reply message for page command. Error is {}", error);
    }
});

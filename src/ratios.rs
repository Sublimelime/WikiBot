extern crate serenity;
extern crate rand;
extern crate json;

use self::serenity::model::Message;
use self::serenity::utils::Colour;
use std::fs::File;
use std::io::prelude::*;

use commands::*;

/// Prints out a grand list of all current stored ratios.
command!(ratios(_context, message) {
    let parsed_json = get_ratio_json();

    let mut embed_content = String::new();

    for entry in parsed_json.entries() {
        //Destructure the tuple to make code easier
        let (key, value) = entry;

        //Change from proprietary format into native
        embed_content += format!("{}\n{}\n\n", key, value.as_str().unwrap()).as_str();
    }

    //Send the message with embed
    let _ = message.channel_id.send_message(|a| a
                                            .content("List of all registered ratios:")
                                            .embed(|b| b
                                                   .field(|c| c.name("List:").value(&embed_content[..]))
                                                   .author(|d| d
                                                           .name("WikiBot")
                                                           .url("https://bitbucket.com/Gangsir")
                                                          )
                                                   .color(Colour::from_rgb(100,200,100))
                                                  ));
});

/// Adds a ratio to the list of current ratios.
/// Can only be used by moderators.
command!(ratio_add(_context, message, _args, name: String, ratio: String) {
    // Reject if they don't use quotes, since the ratio wouldn't be added correctly otherwise
    if !message.content_safe().contains("\"") {
        say_into_chat(&message, format!("I'm sorry, I didn't understand your input correctly.
                                        Use ```{}help ratio add``` for info on how to format this command.", ::PREFIX).as_str());
    } else {
        let mut parsed_json = get_ratio_json();

    }
});

/// Retrieves a ratio from the storage of the bot.
command!(ratio_get(_context, message, _args, ratio: String) {
    let parsed_json = get_ratio_json();

});

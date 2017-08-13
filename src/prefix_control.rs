/// Holds command definitions for manipulating prefixes

use constants::*;
use commands::*;

/// Registers a prefix for the guild that it's used in.
command!(register_prefix(_c, m, args) {
    let prefix = args.join(" ");
    let mut prefixes = PREFIXES.lock().unwrap();
    let guild_id = m.guild_id().unwrap();

    let old_prefix = prefixes.insert(guild_id, prefix.clone());
    if let None = old_prefix {
        if let Err(_) = send_success_embed(&m, format!("Success, registered prefix {}.", prefix).as_str()) {
            say_into_chat(&m, format!("Success, registered prefix {}.", prefix));
        }
    } else {
        let old_prefix = old_prefix.unwrap();
        if let Err(_) = send_success_embed(&m, format!("Success, registered prefix {}, over old prefix {}.", prefix, old_prefix).as_str()) {
            say_into_chat(&m, format!("Success, registered prefix {}, over old prefix {}.", prefix, old_prefix));
        }
    }
    make_log_entry(format!("Changed prefix of server id {:?}, to new prefix {}", guild_id, prefix), "Prefix");
});

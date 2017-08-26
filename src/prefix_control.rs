/// Holds command definitions for manipulating prefixes

extern crate json;

use constants::*;
use common_funcs::*;

/// Registers a prefix for the guild that it's used in. {{{1
command!(register_prefix(_c, m, args) {
    let prefix = args.join(" ");
    if prefix.is_empty() {
        say_into_chat(&m, "Cannot set prefix to nothing, sorry.");
        return Err(String::from("User did not provide a prefix to register to."));
    }
    {
        let mut prefixes = PREFIXES.lock().unwrap();
        let guild_id = m.guild_id().unwrap();

        let old_prefix = prefixes.insert(guild_id, prefix.clone());
        if let None = old_prefix {
            if let Err(_) = send_success_embed(&m,
                                               format!("Success, registered prefix {}.", prefix)
                                               .as_str()) {
                say_into_chat(&m, format!("Success, registered prefix {}.", prefix));
            }
        } else {
            let old_prefix = old_prefix.unwrap();
            if let Err(_) = send_success_embed(&m,
                                               format!("Success, registered prefix {}, over old prefix {}.",
                                                       prefix,
                                                       old_prefix)
                                               .as_str()) {
                say_into_chat(&m, format!("Success, registered prefix {}, over old prefix {}.", prefix, old_prefix));
            }
        }
    }
    // Edit prefixes file with new prefix
    backup_prefixes();

    make_log_entry(format!("Changed prefix of server id {:?}, to new prefix {}",
                           m.guild_id().unwrap(),
                           prefix),
                           "Prefix");
});

// Tests {{{1
#[cfg(test)]
mod tests {
    extern crate serenity;

    use constants::*;
    use self::serenity::model::GuildId;

    #[test]
    fn guild_can_change_prefix() {
        //Setup vars
        let mut prefixes = PREFIXES.lock().unwrap();
        let id = GuildId::from(222222222222222222);
        let _ = prefixes.insert(id, String::from("-"));

        if let Some(old) = prefixes.insert(id, String::from("+")) {
            assert_eq!(old, "-");
        } else {
            panic!("Failed to realize that a prefix had already been set.")
        }
    }

}

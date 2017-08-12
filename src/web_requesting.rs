/// A file used to hold all commands that make web requests.

extern crate rss;
extern crate serenity;

use commands::*;
use self::rss::Channel;

/// Requests a link to the newest FFF posted by the devs.
command!(fff(_context, message) {
    // Get rss channel from wube's blog rss url
    let channel = Channel::from_url("http://www.factorio.com/blog/rss").unwrap();
});

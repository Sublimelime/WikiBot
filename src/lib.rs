extern crate atom_syndication;
extern crate chrono;
extern crate json;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate serenity;

pub mod common_funcs;
pub mod constants;
mod faq_system;
mod levenshtein;
mod linkmod;
mod prefix_control;
mod recipe_system;
mod simple_commands;
mod web_requesting;

pub mod commands {
    pub use faq_system::{faqs, faq_add, faq_get, faq_delete, faq_deleteall, faq_set};
    pub use linkmod::linkmod;
    pub use prefix_control::register_prefix;
    pub use recipe_system::recipe;
    pub use simple_commands::{ping, stop_process, info, uptime, host, page, fff_old};
    pub use web_requesting::{fff, version};
}
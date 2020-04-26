pub use self::config::Config;

pub use self::errors::*;
pub use self::vars::{Variables, VariablesExt};

mod config;
mod errors;
pub mod event_handler;
mod model;
mod parse;
pub mod run;
mod vars;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

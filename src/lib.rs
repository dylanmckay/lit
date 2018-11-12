pub use self::config::Config;

pub use self::errors::*;

mod config;
mod errors;
mod model;
mod parse;
pub mod run;
pub mod vars;

#[cfg(test)]
mod lit_unit_tests;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate tempfile;
extern crate term;
extern crate walkdir;


pub use self::config::Config;

pub use self::errors::*;

pub mod run;
pub mod vars;

mod model;
mod parse;

mod print;
mod config;
mod errors;

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

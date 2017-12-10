pub use self::config::Config;

use self::test::*;
use self::instance::Instance;
use self::matcher::Matcher;
pub use self::errors::*;

pub mod run;

mod tool;
mod test;
mod find;
mod print;
mod instance;
mod config;
mod matcher;
mod errors;

#[cfg(test)]
mod unit_tests;

extern crate walkdir;
extern crate term;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;

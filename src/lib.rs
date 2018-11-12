pub use self::config::Config;

use self::test::*;
use self::instance::Instance;
pub use self::errors::*;

pub mod run;

mod exec;
mod model;
mod parse;

mod test;
mod find_files;
mod print;
mod instance;
mod config;
mod matcher;
mod errors;

#[cfg(test)]
mod unit_tests;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate tempfile;
extern crate term;
extern crate walkdir;

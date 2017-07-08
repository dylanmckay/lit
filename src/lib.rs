pub use self::config::Config;

use self::test::*;
use self::instance::Instance;
use self::matcher::Matcher;

pub mod run;

mod tool;
mod test;
mod find;
mod print;
mod instance;
mod config;
mod matcher;

#[cfg(test)]
mod unit_tests;

extern crate walkdir;
extern crate term;
extern crate regex;
extern crate argparse;
#[macro_use]
extern crate lazy_static;

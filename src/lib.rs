pub use self::config::Config;

use self::test::*;
use self::instance::Instance;

pub mod run;

mod tool;
mod test;
mod find;
mod print;
mod instance;
mod config;

#[cfg(test)]
mod unit_tests;

extern crate walkdir;
extern crate term;
extern crate regex;
extern crate argparse;

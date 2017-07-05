pub use self::config::Config;

use self::tool::*;
use self::test::*;
use self::instance::Instance;

mod tool;
mod test;
mod find;
mod print;
mod instance;
mod config;

pub mod run;

extern crate walkdir;
extern crate term;
extern crate regex;
extern crate argparse;

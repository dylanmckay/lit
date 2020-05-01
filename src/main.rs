extern crate lit;
extern crate clap;

use clap::{App, Arg, ArgMatches};
use std::env::consts;

fn parse_cmdline() -> ArgMatches<'static> {
    let app = App::new("LLVM-lit inspired generic testing tool")
                          .version(env!("CARGO_PKG_VERSION"))
                          .author(env!("CARGO_PKG_AUTHORS"))
                          .about(env!("CARGO_PKG_DESCRIPTION"))
                          .arg(Arg::with_name("v")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity"));
    let app = lit::config::clap::mount_inside_app(app, true);

    let matches = app.get_matches();
    matches
}

fn main() {
    let arg_matches = parse_cmdline();

    println!("hello");

    lit::run::tests(lit::event_handler::Default::default(), |config| {
        config.add_search_path("integration-tests/");
        config.add_extension("txt");

        config.constants.insert("arch".to_owned(), consts::ARCH.to_owned());
        config.constants.insert("os".to_owned(), consts::OS.to_owned());

        lit::config::clap::parse_arguments(&arg_matches, config);
    }).unwrap()
}

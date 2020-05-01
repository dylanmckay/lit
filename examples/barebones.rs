extern crate lit;

use std::env::consts;

fn main() {
    lit::run::tests(lit::event_handler::Default::default(), |config| {
        config.add_search_path("test/");
        config.add_extension("cpp");

        config.constants.insert("arch".to_owned(), consts::ARCH.to_owned());
        config.constants.insert("os".to_owned(), consts::OS.to_owned());
    }).unwrap()
}

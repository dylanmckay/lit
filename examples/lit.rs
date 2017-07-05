extern crate lit;

use std::env::consts;

fn main() {
    lit::run::tests(|config| {
        config.add_search_path("test/");
        config.add_extension("cpp");

        config.constants.insert("arch", consts::ARCH);
        config.constants.insert("os", consts::OS);
    })
}

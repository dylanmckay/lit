extern crate lit;

fn main() {
    lit::run::tests(|config| {
        config.add_path("test/");
        config.add_extension("cpp");
    })
}

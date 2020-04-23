use crate::run;

const CRATE_PATH: &'static str = env!("CARGO_MANIFEST_DIR");

#[test]
fn thing() {
    run::tests(|config| {
        config.add_search_path(format!("{}/tests", CRATE_PATH));
        config.add_extension("txt");
    }).expect("unit test(s) failed");
}

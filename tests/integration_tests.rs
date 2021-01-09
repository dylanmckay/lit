use lit::run;

const CRATE_PATH: &'static str = env!("CARGO_MANIFEST_DIR");

/// Runs all of the integration tests in the top-level directory
/// of the repository.
#[test]
fn integration_tests() {
    pretty_env_logger::init();

    run::tests(lit::event_handler::Default::default(), |config| {
        config.add_search_path(format!("{}/integration-tests", CRATE_PATH));
        for ext in lit::INTEGRATION_TEST_FILE_EXTENSIONS {
            config.add_extension(ext);
        }
    }).expect("unit test(s) failed");

    // Now run the tests again but use a custom shell instead.
    run::tests(lit::event_handler::Default::default(), |config| {
        config.add_search_path(format!("{}/integration-tests", CRATE_PATH));
        for ext in lit::INTEGRATION_TEST_FILE_EXTENSIONS {
            config.add_extension(ext);
        }

        config.shell = "sh".to_string();
    }).expect("unit test(s) failed");
}

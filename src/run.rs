//! Routines for running tests.

use argparse;

use argparse::ArgumentParser;
use std::borrow::Borrow;

use {Context, Config, print};

/// Runs all tests according to a given config.
///
/// # Parameters
///
/// * `config_fn` is a function which sets up the test config.
pub fn tests<F>(config_fn: F)
    where F: Fn(&mut Config) {
    let mut config = Config::default();
    config_fn(&mut config);

    let mut paths: Vec<String> = config.test_paths.iter().map(|p| p.display().to_string()).collect();

    {
        let mut tmp = false;
        let mut ap = ArgumentParser::new();
        ap.set_description("Runs tests");

        ap.refer(&mut paths)
            .add_argument("paths", argparse::List,
                          r#"Paths to test"#);
        // Required in order to use 'cargo test --nocapture'.
        ap.refer(&mut tmp)
            .add_option(&["--nocapture"], argparse::StoreTrue,
            "ignore this");
        ap.parse_args_or_exit();
    }

    if paths.is_empty() {
        util::abort("no filenames given")
    }

    let paths = paths.iter()
                     .map(|s| s.borrow());

    let test_paths = match ::find::in_paths(paths, &config) {
        Ok(paths) => paths,
        Err(e) => util::abort(format!("could not find files: {}", e)),
    };

    if test_paths.is_empty() {
        print::warning("could not find any tests");
        return;
    }

    let mut context = test_paths.into_iter().fold(Context::new(), |c,file| {
        let test = util::parse_test(&file, &config).unwrap();
        c.test(test)
    });

    match util::crate_dir() {
        Some(dir) => context.add_search_dir(dir),
        None => print::warning("could not find tool directory"),
    }

    let results = context.run(&config);

    for result in results.iter() {
        print::result(result)
    }
}

mod util
{
    use {Test, Config};
    use print;

    use std::error::Error;
    use std::io::Read;
    use std;

    pub fn crate_dir() -> Option<String> {
        let current_exec = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => abort(
                format!("failed to get current executable path: {}", e)),
        };

        current_exec.parent().map(|p| p.to_str().unwrap().to_owned())
    }

    pub fn parse_test(file_name: &str, config: &Config) -> Result<Test,String> {
        let mut text = String::new();
        open_file(file_name).read_to_string(&mut text).unwrap();
        Test::parse(file_name, text.chars(), config)
    }

    fn open_file(path: &str) -> std::fs::File {
        match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => abort(format!("could not open {}: {}",
                                    path, e.description())),
        }
    }
    pub fn abort<S>(msg: S) -> !
        where S: Into<String> {
        print::failure(format!("error: {}", msg.into()));

        std::process::exit(1);
    }
}


use argparse;

use argparse::ArgumentParser;
use std::borrow::Borrow;

use {Context, print};

pub fn tests<F>(config_fn: F)
    where F: Fn(&mut ::Configuration) {
    let mut configuration = ::Configuration::default();
    config_fn(&mut configuration);

    let mut paths: Vec<String> = configuration.test_paths.iter().map(|p| p.display().to_string()).collect();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Runs tests");

        ap.refer(&mut paths)
            .add_argument("paths", argparse::List,
                          r#"Paths to test"#);
        ap.parse_args_or_exit();
    }

    if paths.is_empty() {
        util::abort("no filenames given")
    }

    let paths = paths.iter()
                     .map(|s| s.borrow());

    let test_paths = match ::find::in_paths(paths, &configuration) {
        Ok(paths) => paths,
        Err(e) => util::abort(format!("could not find files: {}", e)),
    };

    if test_paths.is_empty() {
        print::warning("could not find any tests");
        return;
    }

    let mut context = test_paths.into_iter().fold(Context::new(), |c,file| {
        let test = util::parse_test(&file).unwrap();
        c.test(test)
    });

    match util::tool_dir() {
        Some(dir) => context.add_search_dir(dir),
        None => print::warning("could not find tool directory"),
    }

    let results = context.run();

    for result in results.iter() {
        print::result(result)
    }
}

mod util
{
    use Test;
    use print;

    use std::error::Error;
    use std::io::Read;
    use std;

    pub fn tool_dir() -> Option<String> {
        let current_exec = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => abort(
                format!("failed to get current executable path: {}", e)),
        };

        current_exec.parent().map(|p| p.to_str().unwrap().to_owned())
    }

    pub fn parse_test(file_name: &str) -> Result<Test,String> {
        let mut text = String::new();
        open_file(file_name).read_to_string(&mut text).unwrap();
        Test::parse(file_name, text.chars())
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


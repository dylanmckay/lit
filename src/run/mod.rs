//! Routines for running tests.

pub(crate) mod find_files;
mod test_evaluator;

pub use self::test_evaluator::CommandLine;

use crate::{Config, event_handler::{EventHandler, TestSuiteDetails}};
use crate::model::*;

/// Runs all tests according to a given config.
///
/// Return `Ok` if all tests pass, and `Err` otherwise.
///
/// # Parameters
///
/// * `config_fn` is a function which sets up the test config.
/// * `event_handler` is an object which presents the user interface to the user.
///
pub fn tests<F>(
    mut event_handler: impl EventHandler,
    config_fn: F,
    ) -> Result<(), ()>
    where F: Fn(&mut Config) {
    let mut config = Config::default();
    config_fn(&mut config);

    if config.test_paths.is_empty() {
        util::abort("no test paths given to lit")
    }

    let test_paths = match find_files::with_config(&config) {
        Ok(paths) => paths,
        Err(e) => util::abort(format!("could not find test files: {}", e)),
    };

    if test_paths.is_empty() {
        event_handler.note_warning("could not find any tests");
        return Err(());
    }

    let test_suite_details = TestSuiteDetails {
        number_of_test_files: test_paths.len(),
    };

    event_handler.on_test_suite_started(&test_suite_details, &config);

    let mut has_failure = false;
    for test_file_path in test_paths {
        let test_file = util::parse_test(&test_file_path).unwrap();
        let is_successful = self::single_file(&test_file, &mut event_handler, &config);

        if !is_successful { has_failure = true; }
    }

    event_handler.on_test_suite_finished(!has_failure, &config);

    if !has_failure { Ok(()) } else { Err(()) }
}

/// Executes a single, parsed test file.
///
/// Returns `true` if all the tests in the file succeeded.
fn single_file(
    test_file: &TestFile,
    event_handler: &mut dyn EventHandler,
    config: &Config,
    ) -> bool {
    let test_results = test_evaluator::execute_tests(test_file, config);

    // The overall result is failure if there are any failures, otherwise it is a pass.
    let overall_result = test_results.iter().map(|(r, _, _, _)| r).filter(|r| r.is_erroneous()).next().cloned().unwrap_or(TestResultKind::Pass);

    let result = TestResult {
        path: test_file.path.clone(),
        overall_result,
        individual_run_results: test_results.into_iter().map(|(a, b, c, d)| (a, b.clone(), c, d)).collect(),
    };

    let is_erroneous = result.overall_result.is_erroneous();

    event_handler.on_test_finished(result, config);

    !is_erroneous
}

mod util
{
    use crate::model::*;
    use crate::parse;

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

    pub fn parse_test(file_name: &str) -> Result<TestFile,String> {
        let mut text = String::new();
        open_file(file_name).read_to_string(&mut text).unwrap();
        parse::test_file(file_name, text.chars())
    }

    fn open_file(path: &str) -> std::fs::File {
        match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => abort(format!("could not open {}: {}",
                                    path, e.to_string())),
        }
    }
    pub fn abort<S>(msg: S) -> !
        where S: Into<String> {
        eprintln!("error: {}", msg.into());

        std::process::exit(1);
    }
}

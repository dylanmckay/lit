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

    // Used for storing artifacts generated during testing.
    let artifact_config = save_artifacts::Config {
        artifacts_dir: config.save_artifacts_to_directory.clone(),
    };

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
        let test_file = util::parse_test(test_file_path).unwrap();
        let is_successful = self::single_file(&test_file, &mut event_handler, &config, &artifact_config);

        if !is_successful { has_failure = true; }
    }
    let is_successful = !has_failure;

    event_handler.on_test_suite_finished(is_successful, &config);
    save_artifacts::suite_status(is_successful, &artifact_config);

    if !has_failure { Ok(()) } else { Err(()) }
}

/// Executes a single, parsed test file.
///
/// Returns `true` if all the tests in the file succeeded.
fn single_file(
    test_file: &TestFile,
    event_handler: &mut dyn EventHandler,
    config: &Config,
    artifact_config: &save_artifacts::Config,
    ) -> bool {
    let test_results = test_evaluator::execute_tests(test_file, config);

    // The overall result is failure if there are any failures, otherwise it is a pass.
    let overall_result = test_results.iter().map(|(r, _, _, _)| r).filter(|r| r.is_erroneous()).next().cloned().unwrap_or(TestResultKind::Pass);

    let result = TestResult {
        path: test_file.path.clone(),
        overall_result,
        individual_run_results: test_results.into_iter().map(|(a, b, c, d)| (a, b.clone(), c, d)).collect(),
    };

    save_artifacts::run_results(&result, test_file, artifact_config);

    let is_erroneous = result.overall_result.is_erroneous();

    event_handler.on_test_finished(result, config);

    !is_erroneous
}

mod util
{
    use crate::model::*;
    use crate::parse;

    use std::{io::Read, path::Path};
    use std;

    pub fn crate_dir() -> Option<String> {
        let current_exec = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => abort(
                format!("failed to get current executable path: {}", e)),
        };

        current_exec.parent().map(|p| p.to_str().unwrap().to_owned())
    }

    pub fn parse_test(path: TestFilePath) -> Result<TestFile, String> {
        let mut text = String::new();
        open_file(&path.absolute).read_to_string(&mut text).unwrap();
        parse::test_file(path, text.chars())
    }

    fn open_file(path: &Path) -> std::fs::File {
        match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => abort(format!("could not open {}: {}",
                                    path.display(), e.to_string())),
        }
    }
    pub fn abort<S>(msg: S) -> !
        where S: Into<String> {
        eprintln!("error: {}", msg.into());

        std::process::exit(1);
    }
}

mod save_artifacts {
    use super::CommandLine;
    use crate::model::*;
    use std::path::{Path, PathBuf};
    use std::fs;

    const SUITE_STATUS_PATH: &'static str = "suite-status.txt";

    #[derive(Clone, Debug)]
    pub struct Config {
        pub artifacts_dir: Option<PathBuf>,
    }

    pub fn suite_status(is_successful: bool, config: &Config) {
        save(&Path::new(SUITE_STATUS_PATH), config, || {
            if is_successful {
                "successful\n"
            } else {
                "failed\n"
            }
        });
    }

    pub fn run_results(test_result: &TestResult, test_file: &TestFile, artifact_config: &Config) {
        let only_one_run_command = test_result.individual_run_results.len() == 1;

        for (i, (result_kind, _, command_line, output)) in test_result.individual_run_results.iter().enumerate() {
            let run_number = if only_one_run_command { None } else { Some(i + 1) };
            self::individual_run_result(run_number, result_kind, command_line, output, test_file, artifact_config);
        }
    }

    pub fn individual_run_result(run_number: Option<usize>, result_kind: &TestResultKind, command_line: &CommandLine, output: &ProgramOutput, test_file: &TestFile, config: &Config) {
        let test_file_extension = test_file.path.absolute.extension().and_then(|s| s.to_str()).unwrap_or("txt");

        let dir_run_result = match run_number {
            Some(run_number) => test_file.path.relative.join(format!("run-command-{}", run_number)),
            None => test_file.path.relative.clone(),
        };

        save(&dir_run_result.join("result.txt"), config, || {
            format!("{:#?}\n", result_kind)
        });

        save(&dir_run_result.join("stdout.txt"), config, || &output.stdout[..]);
        save(&dir_run_result.join("stderr.txt"), config, || &output.stderr[..]);
        save(&dir_run_result.join("command-line.txt"), config, || format!("{}\n", command_line.0));

        save(&dir_run_result.join(&format!("copy-of-test-case.{}", test_file_extension)), config, || std::fs::read(&test_file.path.absolute).unwrap());

        create_symlink(&test_file.path.absolute, &dir_run_result.join(&format!("symlink-to-test-case.{}", test_file_extension)), config)
    }

    fn save<C>(relative_path: &Path, config: &Config, render: impl FnOnce() -> C )
        where C: AsRef<[u8]> {
        if let Some(artifacts_dir) = config.artifacts_dir.as_ref() {
            let absolute_path = artifacts_dir.join(relative_path);
            let parent_directory = absolute_path.parent().unwrap();

            let file_content = render();

            fs::create_dir_all(parent_directory).unwrap();
            fs::write(absolute_path, file_content).unwrap();
        }
    }

    /// Creates a symlink, unless symlinks are not supported in this environment.
    fn create_symlink(src: &Path, relative_dst: &Path, config: &Config) {
        #[cfg(unix)]
        fn create_symlink_impl(src: &Path, dst: &Path) -> std::io::Result<()> { std::os::unix::fs::symlink(src, dst) }
        #[cfg(not(unix))]
        fn create_symlink_impl(_: &Path, _: &Path) -> std::io::Result<()> { Ok(()) }

        if let Some(artifacts_dir) = config.artifacts_dir.as_ref() {
            let dst = artifacts_dir.join(relative_dst);

            if dst.exists() {
                fs::remove_file(&dst).unwrap(); // Remove the symlink.
            }
            create_symlink_impl(src, &dst).unwrap();
        }

    }
}

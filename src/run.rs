//! Routines for running tests.

use {Instance, Config, print};
use model::*;

#[derive(Clone,Debug,PartialEq,Eq)]
struct Context
{
    pub exec_search_dirs: Vec<String>,
    pub test_files: Vec<TestFile>,
}

/// Runs all tests according to a given config.
///
/// Return `Ok` if all tests pass, and `Err` otherwise.
///
/// # Parameters
///
/// * `config_fn` is a function which sets up the test config.
pub fn tests<F>(config_fn: F) -> Result<(), ()>
    where F: Fn(&mut Config) {
    let mut config = Config::default();
    config_fn(&mut config);

    if config.test_paths.is_empty() {
        util::abort("no test paths given to lit")
    }

    let test_paths = match ::find_files::with_config(&config) {
        Ok(paths) => paths,
        Err(e) => util::abort(format!("could not find files: {}", e)),
    };

    if test_paths.is_empty() {
        print::warning("could not find any tests");
        return Err(());
    }

    let mut context = test_paths.into_iter().fold(Context::new(), |c,file| {
        let test = util::parse_test(&file).unwrap();
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

    let has_failure = results.iter().any(|r| {
        if let TestResultKind::Fail { .. } = r.kind { true } else { false }
    });
    if !has_failure { Ok(()) } else { Err(()) }
}

pub fn test_file(test_file: &TestFile, config: &Config) -> TestResult {
    if test_file.is_empty() {
        return TestResult {
            path: test_file.path.clone(),
            kind: TestResultKind::Skip,
        }
    }

    for instance in create_instances(&test_file) {
        let kind = instance.run(test_file, config);

        match kind {
            TestResultKind::Pass => continue,
            TestResultKind::Skip => {
                return TestResult {
                    path: test_file.path.clone(),
                    kind: TestResultKind::Pass,
                }
            },
            _ => {
                return TestResult {
                    path: test_file.path.clone(),
                    kind,
                }
            },
        }
    }

    TestResult {
        path: test_file.path.clone(),
        kind: TestResultKind::Pass,
    }
}

fn create_instances(test_file: &TestFile) -> Vec<Instance> {
    test_file.directives.iter().flat_map(|directive| {
        if let Command::Run(ref invocation) = directive.command {
            Some(Instance::new(invocation.clone()))
        } else {
            None
        }
    }).collect()
}


mod util
{
    use model::*;
    use {parse, print};

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

    pub fn parse_test(file_name: &str) -> Result<TestFile,String> {
        let mut text = String::new();
        open_file(file_name).read_to_string(&mut text).unwrap();
        parse::test_file(file_name, text.chars())
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

impl Context
{
    pub fn new() -> Self {
        Context {
            exec_search_dirs: Vec::new(),
            test_files: Vec::new(),
        }
    }

    pub fn test(mut self, test_file: TestFile) -> Self {
        self.test_files.push(test_file);
        self
    }

    pub fn run(&self, config: &Config) -> Results {
        let test_results = self.test_files.iter().map(|test_file| {
            self::test_file(test_file, config)
        }).collect();

        Results { test_results }
    }

    pub fn add_search_dir(&mut self, dir: String) {
        self.exec_search_dirs.push(dir);
    }
}


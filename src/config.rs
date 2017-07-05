use std::path::PathBuf;

/// The configuration of the test runner.
#[derive(Clone, Debug)]
pub struct Config
{
    /// A list of file extensions which contain tests.
    pub supported_file_extensions: Vec<String>,
    /// Paths to tests or folders containing tests.
    pub test_paths: Vec<PathBuf>,
}

impl Config
{
    /// Marks a file extension as supported by the runner.
    ///
    /// We only attempt to run tests for files within the extension
    /// whitelist.
    pub fn add_extension<S>(&mut self, ext: S) where S: Into<String> {
        self.supported_file_extensions.push(ext.into())
    }

    /// Adds a search path to the test runner.
    ///
    /// We will recurse through the path to find tests.
    pub fn add_search_path<P>(&mut self, path: P) where P: Into<String> {
        self.test_paths.push(PathBuf::from(path.into()));
    }

    /// Checks if a given extension will have tests run on it
    pub fn is_extension_supported(&self, extension: &str) -> bool {
        self.supported_file_extensions.iter().
            find(|ext| &ext[..] == extension).is_some()
    }
}

impl Default for Config
{
    fn default() -> Self {
        Config {
            supported_file_extensions: Vec::new(),
            test_paths: Vec::new(),
        }
    }
}

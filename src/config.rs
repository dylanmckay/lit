use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Configuration
{
    /// A list of file extensions which contain tests.
    pub supported_file_extensions: Vec<String>,
    /// Paths to tests or folders containing tests.
    pub test_paths: Vec<PathBuf>,
}

impl Configuration
{
    pub fn add_extension<S>(&mut self, ext: S) where S: Into<String> {
        self.supported_file_extensions.push(ext.into())
    }

    pub fn add_path<P>(&mut self, path: P) where P: Into<String> {
        self.test_paths.push(PathBuf::from(path.into()));
    }

    pub fn is_extension_supported(&self, extension: &str) -> bool {
        self.supported_file_extensions.iter().find(|ext| &ext[..] == extension).is_some()
    }
}

impl Default for Configuration
{
    fn default() -> Self {
        Configuration {
            supported_file_extensions: Vec::new(),
            test_paths: Vec::new(),
        }
    }
}

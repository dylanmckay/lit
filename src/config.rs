#[derive(Clone, Debug)]
pub struct Configuration
{
    /// A list of file extensions which contain tests.
    pub supported_file_extensions: Vec<String>,
}

impl Configuration
{
    pub fn is_extension_supported(&self, extension: &str) -> bool {
        self.supported_file_extensions.iter().find(|ext| &ext[..] == extension).is_some()
    }
}

impl Default for Configuration
{
    fn default() -> Self {
        Configuration {
            supported_file_extensions: Vec::new(),
        }
    }
}

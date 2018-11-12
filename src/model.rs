use Error;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// A tool invocation.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Invocation
{
    /// The original command string.
    pub original_command: String,
}

// TODO: rename to TestFile
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct TestFile
{
    /// The on-disk path to the test file.
    pub path: PathBuf,
    pub directives: Vec<Directive>,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Directive
{
    pub command: Command,
    pub line: u32,
}

#[derive(Clone,Debug)]
pub enum Command
{
    /// Run an external tool.
    Run(Invocation),
    /// Verify that the output text matches an expression.
    Check(TextPattern),
    /// Verify that the very next output line matches an expression.
    CheckNext(TextPattern),
    /// Mark the test as supposed to fail.
    XFail,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct TextPattern {
    pub components: Vec<PatternComponent>,
}

/// A component in a text pattern.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum PatternComponent {
    Text(String),
    Variable(String),
    Regex(String),
    NamedRegex { name: String, regex: String },
}

#[derive(Debug)]
pub enum TestResultKind
{
    Pass,
    UnexpectedPass,
    Error(Error),
    Fail {
        message: String,
        stderr: Option<String>,
    },
    ExpectedFailure,
    Skip,
}

#[derive(Debug)]
pub struct TestResult
{
    pub path: PathBuf,
    pub kind: TestResultKind,
}

#[derive(Debug)]
pub struct Results
{
    pub test_results: Vec<TestResult>,
}

impl PartialEq for Command {
    fn eq(&self, other: &Command) -> bool {
        match *self {
            Command::Run(ref a) => if let Command::Run(ref b) = *other { a == b } else { false },
            Command::Check(ref a) => if let Command::Check(ref b) = *other { a.to_string() == b.to_string() } else { false },
            Command::CheckNext(ref a) => if let Command::CheckNext(ref b) = *other { a.to_string() == b.to_string() } else { false },
            Command::XFail => *other == Command::XFail,
        }
    }
}

impl Eq for Command { }

impl fmt::Display for TextPattern {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for component in self.components.iter() {
            match *component {
                PatternComponent::Text(ref text) => write!(fmt, "{}", text)?,
                PatternComponent::Variable(ref name) => write!(fmt, "$${}", name)?,
                PatternComponent::Regex(ref regex) => write!(fmt, "[[{}]]", regex)?,
                PatternComponent::NamedRegex { ref name, ref regex } => write!(fmt, "[[{}:{}]]", name, regex)?,
            }
        }

        Ok(())
    }
}

impl Directive
{
    pub fn new(command: Command, line: u32) -> Self {
        Directive {
            command: command,
            line: line,
        }
    }
}

impl Results
{
    pub fn test_results(&self) -> ::std::slice::Iter<TestResult> {
        self.test_results.iter()
    }

    pub fn iter(&self) -> ::std::slice::Iter<TestResult> {
        self.test_results()
    }
}

impl TestFile
{
    /// Extra test-specific variables.
    pub fn variables(&self) -> HashMap<String, String> {
        let mut v = HashMap::new();
        v.insert("file".to_owned(), self.path.to_str().unwrap().to_owned());
        v
    }

    pub fn is_empty(&self) -> bool {
        self.directives.is_empty()
    }
}


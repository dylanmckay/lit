use crate::{Error, Variables};
use std::{fmt, path::PathBuf};

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
    pub commands: Vec<Command>,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Command
{
    pub line_number: u32,
    pub kind: CommandKind,
}

#[derive(Clone,Debug)]
pub enum CommandKind
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextPattern {
    pub components: Vec<PatternComponent>,
}

/// A component in a text pattern.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternComponent {
    Text(String),
    Variable(String),
    Regex(String),
    NamedRegex { name: String, regex: String },
}

#[derive(Debug)]
#[must_use]
pub enum TestResultKind
{
    Pass,
    UnexpectedPass,
    Error(Error),
    Fail {
        reason: TestFailReason,
        hint: Option<String>,
    },
    ExpectedFailure,
    Skip,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestFailReason {
    CheckFailed(CheckFailureInfo),
}

/// Information about a failed check in a test.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckFailureInfo {
    pub complete_output_text: String,
    pub successfully_checked_until_byte_index: usize,
    pub expected_pattern: TextPattern,
}

/// Results from executing a test.
#[derive(Debug)]
pub struct TestResult
{
    /// A path to the test.
    pub path: PathBuf,
    /// The kind of result.
    pub kind: TestResultKind,
}

#[derive(Debug)]
pub struct Results
{
    pub test_results: Vec<TestResult>,
}

impl PartialEq for CommandKind {
    fn eq(&self, other: &CommandKind) -> bool {
        match *self {
            CommandKind::Run(ref a) => if let CommandKind::Run(ref b) = *other { a == b } else { false },
            CommandKind::Check(ref a) => if let CommandKind::Check(ref b) = *other { a.to_string() == b.to_string() } else { false },
            CommandKind::CheckNext(ref a) => if let CommandKind::CheckNext(ref b) = *other { a.to_string() == b.to_string() } else { false },
            CommandKind::XFail => *other == CommandKind::XFail,
        }
    }
}

impl Eq for CommandKind { }

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

impl Command
{
    pub fn new(kind: CommandKind, line_number: u32) -> Self {
        Command { kind, line_number }
    }
}

impl TestResultKind {
    /// Checks if the result is considered an error.
    pub fn is_erroneous(&self) -> bool {
        use self::TestResultKind::*;

        match *self {
            UnexpectedPass | Error(..) | Fail { .. } => true,
            Pass | Skip | ExpectedFailure => false,
        }
    }

    pub fn unwrap(&self) {
        if self.is_erroneous() {
            panic!("error whilst running test: {:?}", self);
        }
    }
}

impl CheckFailureInfo {
    /// Gets the slice containing the portion of successfully checked text.
    pub fn successfully_checked_text(&self) -> &str {
        let byte_subslice = &self.complete_output_text.as_bytes()[0..self.successfully_checked_until_byte_index];
        convert_bytes_to_str(byte_subslice)
    }

    /// Gets the slice containing the portion of unchecked, remaining text.
    pub fn remaining_text(&self) -> &str {
        let byte_subslice = &self.complete_output_text.as_bytes()[self.successfully_checked_until_byte_index..];
        convert_bytes_to_str(byte_subslice)
    }
}

impl TestFile
{
    /// Extra test-specific variables.
    pub fn variables(&self) -> Variables {
        let mut v = Variables::new();
        v.insert("file".to_owned(), self.path.to_str().unwrap().to_owned());
        v
    }

    /// Gets an iterator over all `RUN` commands in the test file.
    pub fn run_command_invocations(&self) -> impl Iterator<Item=&Invocation> {
        self.commands.iter().filter_map(|c| match c.kind {
            CommandKind::Run(ref invocation) => Some(invocation),
            _ => None,
        })
    }
}

/// Build a text pattern from a single component.
impl From<PatternComponent> for TextPattern {
    fn from(component: PatternComponent) -> Self {
        TextPattern { components: vec![component] }
    }
}

impl std::fmt::Debug for CheckFailureInfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[derive(Debug)]
        struct CheckFailureInfo<'a> {
            expected_pattern: &'a TextPattern,
            successfully_checked_text: PrintStrTruncate<'a>,
            remaining_text: PrintStrTruncate<'a>,
        }

        const TRUNCATE_MIN: usize = 70;
        const TRUNCATE_MARKER: &'static str = "...";
        struct PrintStrTruncate<'a>(&'a str);
        impl<'a> std::fmt::Debug for PrintStrTruncate<'a> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                if self.0.len() <= TRUNCATE_MIN {
                    std::fmt::Debug::fmt(self.0, fmt)
                } else {
                    let substr = &self.0[0..TRUNCATE_MIN];
                    substr.fmt(fmt)?;
                    std::fmt::Display::fmt(TRUNCATE_MARKER, fmt)
                }
            }
        }

        CheckFailureInfo {
            expected_pattern: &self.expected_pattern,
            remaining_text: PrintStrTruncate(self.remaining_text()),
            successfully_checked_text: PrintStrTruncate(self.successfully_checked_text()),
        }.fmt(fmt)
    }
}

fn convert_bytes_to_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("invalid UTF-8 in output stream")
}


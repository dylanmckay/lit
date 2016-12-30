use Instance;

use tool;
use std;

use regex::{Regex, Captures};

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Directive
{
    pub command: Command,
    pub line: u32,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Command
{
    Run(tool::Invocation),
    Check(Regex),
    CheckNext(Regex),
}

impl Directive
{
    pub fn new(command: Command, line: u32) -> Self {
        Directive {
            command: command,
            line: line,
        }
    }

    /// Converts a match string to a regex.
    ///
    /// Converts from the `[[capture_name:capture_regex]]` syntaxs to
    /// a regex.
    fn parse_regex(mut string: String) -> Regex {
        let capture_regex = Regex::new("\\[\\[(\\w+):(.*)\\]\\]").unwrap();

        if capture_regex.is_match(&string) {
            string = capture_regex.replace_all(&string, |caps: &Captures| {
                format!("(?P<{}>{})", caps.at(1).unwrap(), caps.at(2).unwrap())
            });
        }

        Regex::new(&string).unwrap()
    }

    pub fn maybe_parse(string: &str, line: u32) -> Option<Result<Self,String>> {
        let regex = Regex::new("([A-Z-]+):(.*)").unwrap();

        if !regex.is_match(string) { return None; }

        let captures = regex.captures(string).unwrap();
        let command_str = captures.at(1).unwrap().trim();
        let after_command_str = captures.at(2).unwrap().trim();

        match command_str {
            // FIXME: better message if we have 'RUN :'
            "RUN" => {
                let inner_words = after_command_str.split_whitespace();
                let invocation = match tool::Invocation::parse(inner_words) {
                    Ok(i) => i,
                    Err(e) => return Some(Err(e)),
                };

                Some(Ok(Directive::new(Command::Run(invocation), line)))
            },
            "CHECK" => {
                let regex = Self::parse_regex(after_command_str.to_owned());
                Some(Ok(Directive::new(Command::Check(regex), line)))
            },
            "CHECK-NEXT" => {
                let regex = Self::parse_regex(after_command_str.to_owned());
                Some(Ok(Directive::new(Command::CheckNext(regex), line)))
            },
            _ => {
                Some(Err(format!("command '{}' not known", command_str)))
            },
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Test
{
    pub path: String,
    pub directives: Vec<Directive>,
}

impl Test
{
    pub fn parse<S,I>(name: S, chars: I) -> Result<Self,String>
        where S: Into<String>, I: Iterator<Item=char> {
        let mut directives = Vec::new();
        let test_body: String = chars.collect();

        for (line_idx, line) in test_body.lines().enumerate() {
            let line_number = line_idx + 1;

            match Directive::maybe_parse(line, line_number as _) {
                Some(Ok(directive)) => directives.push(directive),
                Some(Err(e)) => {
                    return Err(format!(
                        "could not parse directive: {}", e)
                    );
                },
                None => continue,
            }
        }

        Ok(Test {
            path: name.into(),
            directives: directives,
        })
    }

    pub fn run(&self, context: &Context) -> TestResult {
        if self.is_empty() {
            return TestResult {
                path: self.path.clone(),
                kind: TestResultKind::Skip,
            }
        }

        for instance in self.instances() {
            let kind = instance.run(self, context);

            match kind {
                TestResultKind::Pass => continue,
                TestResultKind::Skip => {
                    return TestResult {
                        path: self.path.clone(),
                        kind: TestResultKind::Pass,
                    }
                },
                TestResultKind::Fail(msg, desc) => {
                    return TestResult {
                        path: self.path.clone(),
                        kind: TestResultKind::Fail(msg, desc),
                    }
                },
            }
        }

        TestResult {
            path: self.path.clone(),
            kind: TestResultKind::Pass,
        }
    }

    pub fn instances(&self) -> Vec<Instance> {
        self.directives.iter().flat_map(|directive| {
            if let Command::Run(ref invocation) = directive.command {
                Some(Instance::new(invocation.clone()))
            } else {
                None
            }
        }).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.directives.is_empty()
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum TestResultKind
{
    Pass,
    Fail(String, String),
    Skip,
}

impl TestResultKind
{
    pub fn fail<S: Into<String>>(s: S) -> Self {
        TestResultKind::Fail(s.into(), "".to_owned())
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct TestResult
{
    pub path: String,
    pub kind: TestResultKind,
}

impl TestResult
{
    pub fn passed(&self) -> bool {
        if let TestResultKind::Pass = self.kind { true } else { false }
    }

    pub fn failed(&self) -> bool {
        if let TestResultKind::Fail(..) = self.kind { true } else { false }
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Context
{
    pub exec_search_dirs: Vec<String>,
    pub tests: Vec<Test>,
}

impl Context
{
    pub fn new() -> Self {
        Context {
            exec_search_dirs: Vec::new(),
            tests: Vec::new(),
        }
    }

    pub fn test(mut self, test: Test) -> Self {
        self.tests.push(test);
        self
    }

    pub fn run(&self) -> Results {
        let test_results = self.tests.iter().map(|test| {
            test.run(self)
        }).collect();

        Results {
            test_results: test_results,
        }
    }

    pub fn add_search_dir(&mut self, dir: String) {
        self.exec_search_dirs.push(dir);
    }

    pub fn find_in_search_dir(&self, path: &str)
        -> Option<String> {
        for dir in self.exec_search_dirs.iter() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let cur_path = entry.path();

                if std::fs::metadata(&cur_path).unwrap().is_file() {
                    if cur_path.file_name().unwrap() == path {
                        return Some(cur_path.to_str().unwrap().to_owned());
                    }
                }
            }
        }
        None
    }

    pub fn executable_path(&self, path: &str) -> String {
        match self.find_in_search_dir(path) {
            Some(p) => p,
            None => path.to_owned(),
        }
    }
}


#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Results
{
    test_results: Vec<TestResult>,
}

impl Results
{
    pub fn test_results(&self) -> ::std::slice::Iter<TestResult> {
        self.test_results.iter()
    }

    pub fn iter(&self) -> ::std::slice::Iter<TestResult> {
        self.test_results()
    }

    pub fn passed(&self) -> bool {
        self.test_results().all(TestResult::passed)
    }

    pub fn failed(&self) -> bool {
        self.test_results().any(TestResult::failed)
    }
}

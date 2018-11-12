use {Instance, Config};
use model::*;

use std::collections::HashMap;

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Context
{
    pub exec_search_dirs: Vec<String>,
    pub tests: Vec<Test>,
}

#[derive(Debug)]
pub struct Results
{
    test_results: Vec<TestResult>,
}

impl Test
{
    pub fn run(&self, config: &Config) -> TestResult {
        if self.is_empty() {
            return TestResult {
                path: self.path.clone(),
                kind: TestResultKind::Skip,
            }
        }

        for instance in self.instances() {
            let kind = instance.run(self, config);

            match kind {
                TestResultKind::Pass => continue,
                TestResultKind::Skip => {
                    return TestResult {
                        path: self.path.clone(),
                        kind: TestResultKind::Pass,
                    }
                },
                _ => {
                    return TestResult {
                        path: self.path.clone(),
                        kind,
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

impl Directive
{
    pub fn new(command: Command, line: u32) -> Self {
        Directive {
            command: command,
            line: line,
        }
    }

    /// Checks if a strint is a directive.
    pub fn is_directive(string: &str) -> bool {
        DIRECTIVE_REGEX.is_match(string)
    }

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

    pub fn run(&self, config: &Config) -> Results {
        let test_results = self.tests.iter().map(|test| {
            test.run(config)
        }).collect();

        Results {
            test_results: test_results,
        }
    }

    pub fn add_search_dir(&mut self, dir: String) {
        self.exec_search_dirs.push(dir);
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

#[cfg(test)]
mod test {
    use super::*;

    fn parse(line: &str) -> Result<Directive, String> {
        Directive::maybe_parse(line, 0).unwrap()
    }

    #[test]
    fn can_parse_run() {
        let _d = parse("; RUN: foo").unwrap();
    }
}


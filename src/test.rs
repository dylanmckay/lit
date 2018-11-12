use {Instance, Config};
use model::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Results
{
    pub test_results: Vec<TestResult>,
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


use {Instance, Config};
use model::*;

use std::collections::HashMap;

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


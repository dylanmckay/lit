use {Config, Test, Directive, Command, TestResultKind};
use std::collections::HashMap;
use std::{env, process};
use regex::Regex;

use tool;
use std;

const SHELL: &'static str = "bash";

pub struct Instance
{
    pub invocation: tool::Invocation,
}

struct Checker
{
    lines: Lines,
    variables: HashMap<String, String>,
}

/// Iterator over a set of lines.
struct Lines {
    lines: Vec<String>,
    current: usize,
}

impl Instance
{
    pub fn new(invocation: tool::Invocation) -> Self {
        Instance { invocation: invocation }
    }

    pub fn run(self, test: &Test, config: &Config) -> TestResultKind {
        let mut cmd = self.build_command(test, config);

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    return TestResultKind::Error(
                        format!("shell '{}' does not exist", SHELL).into(),
                    );
                },
                _ => return TestResultKind::Error(e.into()),
            },
        };

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap();

            return TestResultKind::Fail {
                message: format!(
                    "exited with code {}", output.status.code().unwrap()),
                stderr: Some(stderr),
            };
        }

        let stdout = String::from_utf8(output.stdout).unwrap();

        let stdout_lines: Vec<_> = stdout.lines().map(|l| l.trim().to_owned()).collect();
        let stdout: String = stdout_lines.join("\n");

        Checker::new(stdout).run(&test)
    }

    pub fn build_command(&self,
                         test: &Test,
                         config: &Config) -> process::Command {
        let mut constants = config.constants.clone();
        constants.extend(test.extra_constants());

        let command_line = self.invocation.resolve(&constants);

        let mut cmd = process::Command::new("bash");
        cmd.args(&["-c", &command_line]);

        if let Ok(current_exe) = env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let current_path = env::var("PATH").unwrap_or(String::new());
                cmd.env("PATH", format!("{}:{}", parent.to_str().unwrap(), current_path));
            }
        }

        cmd
    }
}

impl Checker
{
    fn new(stdout: String) -> Self {
        Checker {
            lines: stdout.into(),
            variables: HashMap::new(),
        }
    }

    fn run(&mut self, test: &Test) -> TestResultKind {
        for directive in test.directives.iter() {
            match directive.command {
                Command::Run(..) => (),
                Command::Check(ref matcher) => {
                    let regex = matcher.resolve(&self.variables);

                    let beginning_line = self.lines.peek().unwrap_or_else(|| "".to_owned());
                    let matched_line = self.lines.find(|l| regex.is_match(l));

                    if let Some(matched_line) = matched_line {
                        self.process_captures(&regex, &matched_line);
                    } else {
                        let message = format_check_error(test,
                            directive,
                            &format!("could not find match: '{}'", matcher),
                            &beginning_line);
                        return TestResultKind::Fail { message, stderr: None };
                    }
                },
                Command::CheckNext(ref matcher) => {
                    let regex = matcher.resolve(&self.variables);

                    if let Some(next_line) = self.lines.next() {
                        if regex.is_match(&next_line) {
                            self.process_captures(&regex, &next_line);
                        } else {
                            let message = format_check_error(test,
                                directive,
                                &format!("could not find match: '{}'", matcher),
                                &next_line);

                            return TestResultKind::Fail { message, stderr: None };
                        }
                    } else {
                        return TestResultKind::Fail {
                            message: format!("check-next reached the end of file"),
                            stderr: None,
                        };
                    }
                },
            }
        }

        TestResultKind::Pass
    }

    pub fn process_captures(&mut self, regex: &Regex, line: &str) {
        // We shouldn't be calling this function if it didn't match.
        debug_assert_eq!(regex.is_match(line), true);
        let captures = if let Some(captures) = regex.captures(line) {
            captures
        } else {
            return;
        };

        for capture_name in regex.capture_names() {
            // we only care about named captures.
            if let Some(name) = capture_name {
                let captured_value = captures.name(name).unwrap();

                self.variables.insert(name.to_owned(), captured_value.as_str().to_owned());
            }
        }
    }
}

impl Lines {
    pub fn new(lines: Vec<String>) -> Self {
        Lines { lines: lines, current: 0 }
    }

    fn peek(&self) -> Option<<Self as Iterator>::Item> {
        self.next_index().map(|idx| self.lines[idx].clone())
    }

    fn next_index(&self) -> Option<usize> {
        if self.current > self.lines.len() { return None; }

        self.lines[self.current..].iter()
            .position(|l| !Directive::is_directive(l))
            .map(|offset| self.current + offset)
    }
}

impl Iterator for Lines
{
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if let Some(next_index) = self.next_index() {
            self.current = next_index + 1;
            Some(self.lines[next_index].clone())
        } else {
            None
        }
    }
}

impl From<String> for Lines
{
    fn from(s: String) -> Lines {
        Lines::new(s.split("\n").map(ToOwned::to_owned).collect())
    }
}

fn format_check_error(test: &Test,
                      directive: &Directive,
                      msg: &str,
                      next_line: &str) -> String {
    self::format_error(test, directive, msg, next_line)
}

fn format_error(test: &Test,
                directive: &Directive,
                msg: &str,
                next_line: &str) -> String {
    format!("{}:{}: {}\nnext line: '{}'", test.path.display(), directive.line, msg, next_line)
}

#[cfg(test)]
mod test {
    use super::*;

    fn lines(s: &str) -> Vec<String> {
        let lines: Lines = s.to_owned().into();
        lines.collect()
    }

    #[test]
    fn trivial_lines_works_correctly() {
        assert_eq!(lines("hello\nworld\nfoo"), &["hello", "world", "foo"]);
    }

    #[test]
    fn lines_ignores_directives() {
        assert_eq!(lines("; RUN: cat %file\nhello\n; CHECK: foo\nfoo"),
                   &["hello", "foo"]);
    }

    #[test]
    fn lines_can_peek() {
        let mut lines: Lines = "hello\nworld\nfoo".to_owned().into();
        assert_eq!(lines.next(), Some("hello".to_owned()));
        assert_eq!(lines.peek(), Some("world".to_owned()));
        assert_eq!(lines.next(), Some("world".to_owned()));
        assert_eq!(lines.peek(), Some("foo".to_owned()));
        assert_eq!(lines.next(), Some("foo".to_owned()));
    }
}


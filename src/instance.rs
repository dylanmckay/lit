use {Context, Test, Directive, Command, TestResultKind};
use std::process;
use std::collections::HashMap;
use regex::Regex;

use tool;
use std;

pub struct Instance
{
    pub invocation: tool::Invocation,
}

impl Instance
{
    pub fn new(invocation: tool::Invocation) -> Self {
        Instance { invocation: invocation }
    }

    pub fn run(self, test: &Test, context: &Context) -> TestResultKind {
        let exe_path = context.executable_path(&self.invocation.executable);
        let mut cmd = self.build_command(test, context);

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    return TestResultKind::Fail(
                        format!("executable not found: {}",
                                exe_path), "".to_owned());
                },
                _ => {
                    return TestResultKind::Fail(
                        format!("could not execute: '{}', {}",
                                exe_path, e), "".to_owned());
                },
            },
        };

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap();

            return TestResultKind::Fail(format!(
                "{} exited with code {}", exe_path,
                output.status.code().unwrap()),
                stderr
            );
        }

        let stdout = String::from_utf8(output.stdout).unwrap();

        let stdout_lines: Vec<_> = stdout.lines().map(|l| l.trim().to_owned()).collect();
        let stdout: String = stdout_lines.join("\n");

        Checker::new(stdout).run(&test)
    }

    pub fn build_command(&self, test: &Test, context: &Context) -> process::Command {
        let exe_path = context.executable_path(&self.invocation.executable);
        let mut cmd = process::Command::new(&exe_path);

        for arg in self.invocation.arguments.iter() {
            let arg_str = arg.resolve(test);
            cmd.arg(arg_str);
        }

        cmd
    }
}

struct Checker
{
    stdout: String,
    variables: HashMap<String, String>,
}

impl Checker
{
    fn new(stdout: String) -> Self {
        Checker {
            stdout: stdout,
            variables: HashMap::new(),
        }
    }

    fn run(&mut self, test: &Test) -> TestResultKind {
        for directive in test.directives.iter() {
            match directive.command {
                Command::Run(..) => (),
                Command::Check(ref regex) => {
                    let regex = self.resolve_variables(regex.clone());

                    let beginning_line = match self.stdout.lines().next() {
                        Some(l) => l.to_owned(),
                        None => return TestResultKind::fail(
                            format_check_error(test, directive, "reached end of file", "")
                        ),
                    };

                    let mut matched_line = None;
                    let tmp: Vec<_> = self.stdout.lines().map(|l| l.to_owned()).skip_while(|line| {
                        if regex.is_match(&line) {
                            matched_line = Some(line.to_owned());
                            false // stop processing lines
                        } else {
                            true
                        }
                    }).skip(1).collect();
                    self.stdout = tmp.join("\n");

                    if let Some(matched_line) = matched_line {
                        self.process_captures(&regex, &matched_line);
                    } else {
                        return TestResultKind::fail(
                            format_check_error(test,
                                               directive,
                                               &format!("could not find match: '{}'", regex),
                                               &beginning_line,
                            )
                        );
                    }
                },
                Command::CheckNext(ref regex) => {
                    let regex = self.resolve_variables(regex.clone());

                    if let Some(ref next_line) = self.stdout.lines().next().map(|l| l.to_owned()) {
                        if regex.is_match(&next_line) {
                            let lines: Vec<_> = self.stdout.lines().skip(1).map(|l| l.to_owned()).collect();
                            self.stdout = lines.join("\n");

                            self.process_captures(&regex, &next_line);
                        } else {
                            return TestResultKind::fail(
                                format_check_error(test,
                                                   directive,
                                                   &format!("could not find match: '{}'", regex),
                                                   &next_line)
                                );
                        }
                    } else {
                        return TestResultKind::fail(format!("check-next reached the end of file"));
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

                self.variables.insert(name.to_owned(), captured_value.to_owned());
            }
        }
    }

    pub fn resolve_variables(&self, mut regex: Regex) -> Regex {
        for (name, value) in self.variables.iter() {
            let subst_expr = format!("[[{}]]", name);
            let regex_str = format!("{}", regex);
            let regex_str = regex_str.replace(&subst_expr, value);
            regex = Regex::new(&regex_str).unwrap();
        }

        regex
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
    format!("{}:{}: {}\nnext line: '{}'", test.path, directive.line, msg, next_line)
}


use crate::{
    model::{CommandKind, Invocation, TestFile, TestResultKind},
    Config,
    vars,
    VariablesExt,
};
use self::state::TestRunState;
use std::{collections::HashMap, env, fs, process};

mod state;
#[cfg(test)] mod state_tests;

const DEFAULT_SHELL: &'static str = "bash";

/// Responsible for evaluating specific tests and collecting
/// the results.
#[derive(Clone)]
pub struct TestEvaluator
{
    pub invocation: Invocation,
}

pub fn execute_tests(test_file: &TestFile, config: &Config) -> TestResultKind {
    for invocation in test_file.run_command_invocations() {
        let initial_variables = {
            let mut vars = HashMap::new();
            vars.extend(config.constants.clone());
            vars.extend(test_file.variables());
            vars
        };

        let mut test_run_state = TestRunState::new(initial_variables);
        let command = self::build_command(invocation, test_file, config);

        match self::collect_output(command) {
            Ok(output) => {
                test_run_state.append_program_output(&output);
            },
            Err(e) => {
                assert!(e.is_erroneous());
                return e;
            },
        }

        for command in test_file.commands.iter() {
            let test_result = match command.kind {
                CommandKind::Run(..) | // RUN commands are already handled above, in the loop.
                    CommandKind::XFail => { // XFAIL commands are handled separately too.
                        TestResultKind::Pass
                    },
                CommandKind::Check(ref text_pattern) => test_run_state.check(text_pattern, config),
                CommandKind::CheckNext(ref text_pattern) => test_run_state.check_next(text_pattern, config),
            };

            if config.cleanup_temporary_files {
                let tempfile_paths = test_run_state.variables().tempfile_paths();

                for tempfile in tempfile_paths {
                    // Ignore errors, these are tempfiles, they go away anyway.
                    fs::remove_file(tempfile).ok();
                }
            }


            // Early return for failures.
            if test_result.is_erroneous() {
                return test_result;
            }
        }
    }

    TestResultKind::Pass
}

fn collect_output(mut command: process::Command) -> Result<String, TestResultKind> {
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                return Err(TestResultKind::Error(
                    format!("shell '{}' does not exist", DEFAULT_SHELL).into(),
                ));
            },
            _ => return Err(TestResultKind::Error(e.into())),
        },
    };

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();

        return Err(TestResultKind::Fail {
            // message: format!("exited with code {}", output.status.code().unwrap()),
            reason: unimplemented!(),
            hint: None,
        });
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    // FIXME: what about stderr?

    Ok(stdout)
}

/// Builds a command that can be used to execute the process behind a `RUN` directive.
fn build_command(invocation: &Invocation,
                 test_file: &TestFile,
                 config: &Config) -> process::Command {
    let mut variables = config.constants.clone();
    variables.extend(test_file.variables());

    let command_line: String = vars::resolve::invocation(invocation, &config, &mut variables);

    let mut cmd = process::Command::new(DEFAULT_SHELL);
    cmd.args(&["-c", &command_line]);

    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let current_path = env::var("PATH").unwrap_or(String::new());
            cmd.env("PATH", format!("{}:{}", parent.to_str().unwrap(), current_path));
        }
    }

    cmd
}


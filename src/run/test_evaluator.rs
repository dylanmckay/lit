use crate::{
    model::{CommandKind, Invocation, TestFile, TestResultKind, TestFailReason, ProgramOutput},
    Config,
    vars,
    VariablesExt,
};
use self::state::TestRunState;
use std::{collections::HashMap, env, fs, process};

mod state;
#[cfg(test)] mod state_tests;

/// Responsible for evaluating specific tests and collecting
/// the results.
#[derive(Clone)]
pub struct TestEvaluator
{
    pub invocation: Invocation,
}

pub fn execute_tests<'test>(test_file: &'test TestFile, config: &Config) -> Vec<(TestResultKind, &'test Invocation, CommandLine, ProgramOutput)> {
    test_file.run_command_invocations().map(|invocation| {
        let initial_variables = {
            let mut vars = HashMap::new();
            vars.extend(config.constants.clone());
            vars.extend(test_file.variables());
            vars
        };

        let mut test_run_state = TestRunState::new(initial_variables);
        let (command, command_line) = self::build_command(invocation, test_file, config);

        let (program_output, execution_result) = self::collect_output(command, command_line.clone(), config);

        test_run_state.append_program_output(&program_output.stdout);
        test_run_state.append_program_stderr(&program_output.stderr);

        if execution_result.is_erroneous() {
            return (execution_result, invocation, command_line, program_output);
        }

        let overall_test_result_kind = run_test_checks(&mut test_run_state, test_file, config);
        (overall_test_result_kind, invocation, command_line, program_output)
    }).collect()
}

fn run_test_checks(
    test_run_state: &mut TestRunState,
    test_file: &TestFile,
    config: &Config,
) -> TestResultKind {
    let mut check_result = TestResultKind::EmptyTest;

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
            check_result = test_result;
            break;
        } else {
            check_result = TestResultKind::Pass;
        }
    }

    match check_result {
        TestResultKind::Fail { reason, hint } => {
            if test_file.is_expected_failure() {
                TestResultKind::ExpectedFailure { actual_reason: reason }
            } else {
                TestResultKind::Fail { reason, hint}
            }
        },
        r => r,
    }
}

fn collect_output(
    mut command: process::Command,
    command_line: CommandLine,
    config: &Config,
) -> (ProgramOutput, TestResultKind) {
    let mut test_result_kind = TestResultKind::Pass;

    let output = match command.output() {
        Ok(o) => o,
        Err(e) => {
            let error_message = match e.kind() {
                std::io::ErrorKind::NotFound => format!("shell '{}' does not exist", &config.shell).into(),
                _ => e.to_string(),
            };

            return (ProgramOutput::empty(), TestResultKind::Error { message: error_message });
        },
    };

    let program_output = ProgramOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };

    if !output.status.success() {
        test_result_kind = TestResultKind::Fail {
            reason: TestFailReason::UnsuccessfulExecution {
                exit_status: output.status.code().unwrap_or_else(|| if output.status.success() { 0 } else { 1 }),
                program_command_line: command_line.0,
            },
            hint: None,
        };
    }

    (program_output, test_result_kind)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandLine(pub String);

/// Builds a command that can be used to execute the process behind a `RUN` directive.
fn build_command(invocation: &Invocation,
                 test_file: &TestFile,
                 config: &Config) -> (process::Command, CommandLine) {
    let mut variables = config.constants.clone();
    variables.extend(test_file.variables());

    let command_line: String = vars::resolve::invocation(invocation, &config, &mut variables);

    let mut cmd = process::Command::new(&config.shell);
    cmd.args(&["-c", &command_line]);
    cmd.envs(&config.env_variables);

    if !config.extra_executable_search_paths.is_empty() {
        let os_path_separator = if cfg!(windows) { ";" } else { ":" };

        let current_path = env::var("PATH").unwrap_or(String::new());
        let paths_to_inject = config.extra_executable_search_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>();
        let os_path_to_inject = format!("{}{}{}", paths_to_inject.join(os_path_separator), os_path_separator, current_path);

        cmd.env("PATH", os_path_to_inject);
    }

    (cmd, CommandLine(command_line))
}

impl std::fmt::Display for CommandLine {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

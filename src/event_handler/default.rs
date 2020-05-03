use crate::{util, Config, model::*};

use itertools::Itertools;
use std::io;
use std::io::prelude::*;
use term;

/// The default event handler, logging to stdout/stderr.
pub struct EventHandler {
    test_results: Vec<TestResult>,
}

impl EventHandler {
    /// Creates a new default event handler.
    pub fn new() -> Self {
        EventHandler { test_results: Vec::new() }
    }
}

impl std::default::Default for EventHandler {
    fn default() -> Self {
        EventHandler::new()
    }
}

impl super::EventHandler for EventHandler {
    fn on_test_suite_started(&mut self, suite_details: &super::TestSuiteDetails, _: &Config) {
        print::reset_colors(); // our white might not match initial console white. we should be consistent.

        print::line();
        print::horizontal_rule();
        print::textln(format!("Running tests ({} files)", suite_details.number_of_test_files));
        print::horizontal_rule();
        print::line();
    }

    fn on_test_suite_finished(&mut self, passed: bool, config: &Config) {
        // Sort the test results so that they will be consecutive.
        // This is required for itertools group_by used before to work properly.
        self.test_results.sort_by_key(|r| r.overall_result.human_label_pluralized());

        print::line();
        print::textln("finished running tests");
        print::test_suite_status_message(passed, false, &self.test_results);
        print::line();
        print::horizontal_rule();
        print::horizontal_rule();
        print::line();

        if !passed {
            let failed_results = self.test_results.iter().filter(|r| r.overall_result.is_erroneous()).collect::<Vec<_>>();

            print::line();
            print::textln_colored(format!("Failing tests ({}/{}):", failed_results.len(), self.test_results.len()), print::YELLOW);
            print::line();

            for failed_test_result in failed_results.iter() {
                print::with("  ", print::StdStream::Err, print::RED); // indent the errors.
                self::result(failed_test_result, false, config);
            }
        }

        print::test_suite_status_message(passed, true, &self.test_results);

        // 'cargo test' will use the color we last emitted if we don't do this.
        print::reset_colors();
    }

    fn on_test_finished(&mut self, result: TestResult, config: &Config) {
        self::result(&result, true, config);

        self.test_results.push(result);
    }

    fn note_warning(&mut self, message: &str) {
        print::warning(message);
    }
}

pub fn result(result: &TestResult, verbose: bool, config: &Config) {
    match result.overall_result {
        TestResultKind::Pass => {
            print::success(format!("PASS :: {}", result.path.relative.display()));
        },
        TestResultKind::UnexpectedPass => {
            print::failure(format!("UNEXPECTED PASS :: {}", result.path.relative.display()));
        },
        TestResultKind::Skip => {
            print::line();
            print::warning(format!(
                "SKIP :: {} (test does not contain any test commands, perhaps you meant to add a 'CHECK'?)",
                     result.path.relative.display()));
            print::line();
        },
        TestResultKind::Error { ref message } => {
            if verbose { print::line(); }

            print::error(format!("ERROR :: {}", result.path.relative.display()));

            if verbose {
                print::textln(message);

                print::line();
            }
        }
        TestResultKind::Fail { ref reason, ref hint } => {
            if verbose { print::line(); }

            print::failure(format!("FAIL :: {}", result.path.relative.display()));

            // FIXME: improve formatting

            if verbose {
                print::line();
                print::text("test failed: ");
                print::textln_colored(reason.human_summary(), print::RED);
                print::line();
                print::textln(reason.human_detail_message(config));

                if let Some(hint_text) = hint {
                    print::textln(format!("hint: {}", hint_text));
                }

                print::line();
            }
        },
        TestResultKind::ExpectedFailure => {
            print::warning(format!("XFAIL :: {}", result.path.relative.display()));
        },
    }

    if verbose && (result.overall_result.is_erroneous() || config.always_show_stderr) {
        for individual_run_result in result.individual_run_results.iter() {
            let (_, _, command_line, output) = individual_run_result;

            let formatted_stderr = crate::model::format_test_output("stderr", &output.stderr, 1, util::TruncateDirection::Bottom, config);
            if !output.stderr.is_empty() {
                print::textln(format!("NOTE: the program '{}' emitted text on standard error:", command_line));
                print::line();
                print::textln(formatted_stderr);
                print::line();
            }
        }
    }
}

mod print {
    pub use term::color::*;
    use super::*;

    #[derive(Copy, Clone)]
    pub enum StdStream { Out, Err }

    pub fn line() {
        with("\n",
             StdStream::Out,
             term::color::WHITE);
    }

    pub fn horizontal_rule() {
        with("=================================================================\n",
             StdStream::Out,
             term::color::WHITE);
    }

    pub fn textln<S>(msg: S)
        where S: Into<String> {
        text(format!("{}\n", msg.into()))
    }

    pub fn text<S>(msg: S)
        where S: Into<String> {
        with(format!("{}", msg.into()),
             StdStream::Out,
             term::color::WHITE);
    }


    pub fn textln_colored<S>(msg: S, color: u32)
        where S: Into<String> {
        with(format!("{}\n", msg.into()),
             StdStream::Out,
             color);
    }


    pub fn success<S>(msg: S)
        where S: Into<String> {
        with(format!("{}\n", msg.into()),
             StdStream::Out,
             term::color::GREEN);
    }

    pub fn warning<S>(msg: S)
        where S: Into<String> {
        with(format!("{}\n", msg.into()),
             StdStream::Err,
             term::color::YELLOW);
    }

    pub fn error<S>(msg: S)
        where S: Into<String> {
        with(format!("{}\n", msg.into()),
             StdStream::Err,
             term::color::RED);
    }

    pub fn failure<S>(msg: S)
        where S: Into<String> {
        with(format!("{}\n", msg.into()),
             StdStream::Err,
             term::color::MAGENTA);
    }

    pub fn test_suite_status_message(passed: bool, verbose: bool, test_results: &[TestResult]) {
        if verbose {
            self::line();
            self::horizontal_rule();
        }

        if verbose {
            self::textln("Suite Status:");
            self::line();

            for (result_label, corresponding_results) in &test_results.iter().group_by(|r| r.overall_result.human_label_pluralized()) {
                self::textln(format!("  {}: {}", result_label, corresponding_results.count()));
            }

            self::line();
            self::horizontal_rule();
            self::line();
        }

        match passed {
            true => self::success("all tests succeeded"),
            false => self::error("error: tests failed"),
        }
    }

    pub fn with<S>(msg: S,
                   stream: StdStream,
                   color: term::color::Color)
        where S: Into<String> {
        set_color(Some(msg), stream, color);
        reset_colors();
    }

    pub fn set_color<S>(msg: Option<S>,
                        stream: StdStream,
                        color: term::color::Color)
        where S: Into<String> {

        match stream {
            StdStream::Out => {
                if let Some(color_term) = term::stdout().as_mut() {
                    color_term.fg(color).unwrap();

                    if let Some(msg) = msg {
                        write!(color_term, "{}", msg.into()).unwrap();
                    }
                } else {
                    if let Some(msg) = msg {
                        write!(io::stdout(), "{}", msg.into()).unwrap();
                    }
                }
            },
            StdStream::Err => {
                if let Some(color_term) = term::stderr().as_mut() {
                    color_term.fg(color).unwrap();

                    if let Some(msg) = msg {
                        write!(color_term, "{}", msg.into()).unwrap();
                    }
                } else {
                    if let Some(msg) = msg {
                        write!(io::stderr(), "{}", msg.into()).unwrap();
                    }
                }
            },
        }
    }

    pub fn reset_colors() {
        for stream in [StdStream::Out, StdStream::Err].iter().cloned() {
            set_color::<String>(None, stream, term::color::WHITE);
        }
    }
}


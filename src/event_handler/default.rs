use crate::{Config, model::*};

use std::io;
use std::io::prelude::*;
use term;

/// The default event handler, logging to stdout/stderr.
pub struct EventHandler {
    failed_results: Vec<TestResult>,
}

impl EventHandler {
    /// Creates a new default event handler.
    pub fn new() -> Self {
        EventHandler { failed_results: Vec::new() }
    }
}

impl std::default::Default for EventHandler {
    fn default() -> Self {
        EventHandler::new()
    }
}

impl super::EventHandler for EventHandler {
    fn on_test_suite_started(&mut self, _: &Config) { }

    fn on_test_suite_finished(&mut self, passed: bool) {
        print::line();
        print::line();

        match passed {
            true => print::success("all tests succeeded"),
            false => print::error("error: tests failed"),
        }

        // 'cargo test' will use the color we last emitted if we don't do this.
        print::reset_colors();
    }

    fn on_test_finished(&mut self, result: TestResult) {
        self::result(&result, true);

        if result.kind.is_erroneous() {
            self.failed_results.push(result);
        }
    }

    fn note_warning(&mut self, message: &str) {
        print::warning(message);
    }
}

pub fn result(result: &TestResult, verbose: bool) {
    match result.kind {
        TestResultKind::Pass => {
            print::success(format!("PASS :: {}", result.path.display()));
        },
        TestResultKind::UnexpectedPass => {
            print::failure(format!("UNEXPECTED PASS :: {}", result.path.display()));
        },
        TestResultKind::Skip => {
            print::line();
            print::warning(format!(
                "SKIP :: {} (test does not contain any test commands, perhaps you meant to add a 'CHECK'?)",
                     result.path.display()));
            print::line();
        },
        TestResultKind::Error(ref e) => {
            if verbose { print::line(); }

            print::error(format!("ERROR :: {}", result.path.display()));

            if verbose {
                print::text(e.to_string());

                print::line();
            }
        }
        TestResultKind::Fail { ref reason, ref hint } => {
            if verbose { print::line(); }

            print::failure(format!("FAIL :: {}", result.path.display()));

            // FIXME: improve formatting

            if verbose {
                print::text(format!("reason: {:?}", reason));
                print::line();

                if let Some(hint_text) = hint {
                    print::text(format!("hint: {}", hint_text));
                }
            }
        },
        TestResultKind::ExpectedFailure => {
            print::warning(format!("XFAIL :: {}", result.path.display()));
        },
    }
}

mod print {
    use super::*;

    #[derive(Copy, Clone)]
    pub enum StdStream { Out, Err }

    pub fn line() {
        with("\n",
             StdStream::Out,
             term::color::WHITE);
    }

    pub fn text<S>(msg: S)
        where S: Into<String> {
        with(format!("{}\n", msg.into()),
             StdStream::Out,
             term::color::WHITE);
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

    pub fn with<S>(msg: S,
                   stream: StdStream,
                   color: term::color::Color)
        where S: Into<String> {

        match stream {
            StdStream::Out => {
                if let Some(color_term) = term::stdout().as_mut() {
                    color_term.fg(color).unwrap();
                    write!(color_term, "{}", msg.into()).unwrap();
                } else {
                    write!(io::stdout(), "{}", msg.into()).unwrap();
                }
            },
            StdStream::Err => {
                if let Some(color_term) = term::stderr().as_mut() {
                    color_term.fg(color).unwrap();
                    write!(color_term, "{}", msg.into()).unwrap();
                } else {
                    write!(io::stderr(), "{}", msg.into()).unwrap();
                }
            },
        }
    }

    pub fn reset_colors() {
        for stream in [StdStream::Out, StdStream::Err].iter().cloned() {
            with("", stream, term::color::WHITE);
        }
    }
}


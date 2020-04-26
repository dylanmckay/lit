use crate::model::*;

use std::io;
use std::io::prelude::*;
use term;

#[derive(Copy, Clone)]
pub enum StdStream {
    Out,
    Err,

}

pub fn result(result: &TestResult, verbose: bool) {
    match result.kind {
        TestResultKind::Pass => {
            success(format!("PASS :: {}", result.path.display()));
        },
        TestResultKind::UnexpectedPass => {
            failure(format!("UNEXPECTED PASS :: {}", result.path.display()));
        },
        TestResultKind::Skip => {
            line();
            warning(format!(
                "SKIP :: {} (test does not contain any test commands, perhaps you meant to add a 'CHECK'?)",
                     result.path.display()));
            line();
        },
        TestResultKind::Error(ref e) => {
            if verbose { line(); }

            error(format!("ERROR :: {}", result.path.display()));

            if verbose {
                text(e.to_string());

                line();
            }
        }
        TestResultKind::Fail { ref reason, ref hint } => {
            if verbose { line(); }

            failure(format!("FAIL :: {}", result.path.display()));

            // FIXME: improve formatting

            if verbose {
                text(format!("reason: {:?}", reason));
                line();

                if let Some(hint_text) = hint {
                    text(format!("hint: {}", hint_text));
                }
            }
        },
        TestResultKind::ExpectedFailure => {
            warning(format!("XFAIL :: {}", result.path.display()));
        },
    }
}

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


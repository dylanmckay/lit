use {TestResult,TestResultKind};

use std;
use term;

pub fn result(result: &TestResult) {
    match result.kind {
        TestResultKind::Pass => {
            success(format!("PASS :: {}", result.path.display()));
        },
        TestResultKind::Skip => {
            line();
            warning(format!(
                "SKIP :: {} (test does not contain any directives)",
                     result.path.display()));
            line();
        },
        TestResultKind::Error(ref e) => {
            line();

            error(format!("ERROR :: {}", result.path.display()));
            text(e.to_string());

            line();
        }
        TestResultKind::Fail { ref message, ref stderr } => {
            line();

            failure(format!("FAIL :: {}", result.path.display()));
            text(message.clone());

            if let Some(stderr) = stderr.as_ref() {
                // Only print stderr if there was output
                if !stderr.is_empty() {
                    line();
                    text("stderr:");
                    line();
                    text(stderr.clone());
                }
            }
            line();
        },
    }
}


pub fn line() {
    with("\n",
         term::stdout().unwrap(),
         term::color::WHITE);
}

pub fn text<S>(msg: S)
    where S: Into<String> {
    with(format!("{}\n", msg.into()),
         term::stdout().unwrap(),
         term::color::WHITE);
}

pub fn success<S>(msg: S)
    where S: Into<String> {
    with(format!("{}\n", msg.into()),
         term::stdout().unwrap(),
         term::color::GREEN);
}

pub fn warning<S>(msg: S)
    where S: Into<String> {
    with(format!("{}\n", msg.into()),
         term::stderr().unwrap(),
         term::color::YELLOW);
}

pub fn error<S>(msg: S)
    where S: Into<String> {
    with(format!("{}\n", msg.into()),
         term::stderr().unwrap(),
         term::color::RED);
}

pub fn failure<S>(msg: S)
    where S: Into<String> {
    with(format!("{}\n", msg.into()),
         term::stderr().unwrap(),
         term::color::MAGENTA);
}

pub fn with<S,W>(msg: S,
                 mut term: Box<term::Terminal<Output=W>>,
                 color: term::color::Color)
    where S: Into<String>, W: std::io::Write {

    term.fg(color).unwrap();
    write!(term, "{}", msg.into()).unwrap();
}

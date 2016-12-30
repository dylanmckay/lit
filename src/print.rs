use {TestResult,TestResultKind};

use std;
use term;

pub fn result(result: &TestResult) {
    match result.kind {
        TestResultKind::Pass => {
            success(format!("PASS :: {}", result.path));
        },
        TestResultKind::Skip => {
            line();
            warning(format!(
                "SKIP :: {} (test does not contain any directives)",
                     result.path));
            line();
        },
        TestResultKind::Fail(ref msg, ref desc) => {
            line();

            failure(format!("FAIL :: {}", result.path));
            text(msg.clone());

            // Only print stderr if there was output
            if !desc.is_empty() {
                line();
                text("stderr:");
                line();
                text(desc.clone());
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

pub fn failure<S>(msg: S)
    where S: Into<String> {
    with(format!("{}\n", msg.into()),
         term::stderr().unwrap(),
         term::color::RED);
}

pub fn with<S,W>(msg: S,
                 mut term: Box<term::Terminal<Output=W>>,
                 color: term::color::Color)
    where S: Into<String>, W: std::io::Write {

    term.fg(color).unwrap();
    write!(term, "{}", msg.into()).unwrap();
}

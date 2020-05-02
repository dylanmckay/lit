//! The test evaluator implementation, independent of external
//! resources like OS processes.

use crate::{
    Config, Variables,
    model::{self, TestResultKind, TestFailReason, TextPattern},
    vars,
};
use std::collections::HashMap;
use regex::Regex;

/// Byte-index relative to entire stream.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AbsoluteByteIndex(pub usize);

/// Byte-index relative to start of unprocessed stream.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RelativeByteIndex(pub usize);

/// The byte range of a matched pattern.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MatchedRange {
    start: RelativeByteIndex,
    end: RelativeByteIndex,
}

/// Responsible for storing the state of execution for a single `RUN` execution.
#[derive(Debug)]
pub struct TestRunState {
    /// All output bytes emitted by the program.
    complete_output_stream: String,
    /// The current position in the stream at which all prior output has been
    /// successfully checked by the test script.
    current_stream_byte_position: AbsoluteByteIndex,
    /// The stderr portion of the command output. This does not get used by `CHECK`s.
    complete_stderr: String,
    /// A list of available variables to the test script.
    variables: HashMap<String, String>,
}

impl TestRunState {
    pub fn new(initial_variables: HashMap<String, String>) -> Self {
        TestRunState {
            complete_output_stream: String::new(),
            current_stream_byte_position: AbsoluteByteIndex(0),
            complete_stderr: String::new(),
            variables: initial_variables,
        }
    }

    /// Appends output from the inner program.
    pub fn append_program_output(&mut self, output: &str) {
        self.complete_output_stream.extend(output.chars())
    }

    /// Appends stderr output.
    pub fn append_program_stderr(&mut self, stderr: &str) {
        self.complete_stderr.extend(stderr.chars())
    }

    /// Verifies that a text pattern appears subsequently in the stream.
    pub fn check(
        &mut self,
        text_pattern: &TextPattern,
        config: &Config) -> TestResultKind {
        self.check_extended(text_pattern, false, config)
    }

    /// Verifies that the very-next non-whitespace line matches a text pattern.
    pub fn check_next(
        &mut self,
        text_pattern: &TextPattern,
        config: &Config) -> TestResultKind {
        self.check_extended(text_pattern, true, config)
    }

    fn check_extended(
        &mut self,
        text_pattern: &TextPattern,
        require_on_next_line: bool,
        config: &Config) -> TestResultKind {

        self.eat_whitespace();

        let next_relative_matched_range = self.next_unprocessed_byte_index_of(text_pattern, config);

        match next_relative_matched_range {
            Some(matched_range) => {
                // Logic for the CHECK-NEXT directive.
                if require_on_next_line {
                    match self.unprocessed_output_stream().find("\n") {
                        Some(index_of_first_new_line_byte) => {
                            if matched_range.start.0 >= index_of_first_new_line_byte {
                                return TestResultKind::Fail {
                                    reason: TestFailReason::CheckFailed(model::CheckFailureInfo {
                                        complete_output_text: self.complete_output_stream.clone(),
                                        successfully_checked_until_byte_index: self.current_stream_byte_position.0,
                                        expected_pattern: text_pattern.clone(),
                                    }),
                                    hint: Some(format!("found a match for '{}', but it does not appear on the next line, as required by the CHECK-NEXT directive", text_pattern)),
                                };
                            }
                        },
                        None => (), // we are on the last line, no need to verify that explicitly.
                    }
                }

                self.current_stream_byte_position += matched_range.end;

                // No other checks should run against the partial line.
                self.eat_until_end_of_line();

                TestResultKind::Pass
            },
            None => {
                model::TestResultKind::Fail {
                    reason: model::TestFailReason::CheckFailed(model::CheckFailureInfo {
                        complete_output_text: self.complete_output_stream.clone(),
                        successfully_checked_until_byte_index: self.current_stream_byte_position.0,
                        expected_pattern: text_pattern.clone(),
                    }),
                    hint: None,
                }
            },
        }
    }

    pub fn unprocessed_output_bytes(&self) -> &[u8] {
        &self.complete_output_stream.as_bytes()[self.current_stream_byte_position.0..]
    }

    /// Gets all of the non-consumed inner program bytes.
    pub fn unprocessed_output_stream(&self) -> &str {
        convert_bytes_to_str(self.unprocessed_output_bytes())
    }

    /// Gets all variables in scope.
    pub fn variables(&self) -> &Variables { &self.variables }

    fn eat_whitespace(&mut self) {
        if self.unprocessed_output_stream().chars().next().map(char::is_whitespace).unwrap_or(false) {
            let first_nonwhitespace_offset = self.unprocessed_output_stream().chars().take_while(|c| c.is_whitespace()).map(char::len_utf8).sum();
            let first_nonwhitespace_offset = RelativeByteIndex(first_nonwhitespace_offset);

            match first_nonwhitespace_offset {
                // if there are no non-whitespace characters, then there cannot be a match.
                RelativeByteIndex(0) => self.set_position_eof(),
                relative_index => self.current_stream_byte_position += relative_index,
            }
        }
    }

    /// Eats all characters until the end of the current line.
    fn eat_until_end_of_line(&mut self) {
        let unprocessed = self.unprocessed_output_stream();

        match unprocessed.find("\n").map(RelativeByteIndex) {
            Some(new_line_index) => {
                self.current_stream_byte_position += RelativeByteIndex(new_line_index.0 + 1);
            },
            None => self.set_position_eof(), // no more new lines in file.
        }
    }

    /// Gets the index of the next occurrence of the given text pattern.
    ///
    /// N.B. Does not advance the unprocessed stream pointer. This only takes a mutable
    /// reference because of the need to resolve the internal test variable list.
    fn next_unprocessed_byte_index_of(&mut self, text_pattern: &TextPattern, config: &Config)
        -> Option<MatchedRange> {
        let regex = vars::resolve::text_pattern(text_pattern, config, &mut self.variables);
        let output_str = self.unprocessed_output_stream();

        debug!("converting expected text pattern to regex: {:?}", regex);

        match regex.find(output_str) {
            Some(regex_match) => {
                let matched_range = MatchedRange {
                    start: RelativeByteIndex(regex_match.start()),
                    end: RelativeByteIndex(regex_match.end()),
                };

                let new_variables = process_captures(&regex, regex_match.as_str());
                self.variables.extend(new_variables);

                Some(matched_range)
            },
            None => None,
        }
    }

    fn set_position_eof(&mut self) {
        let output_bytes = self.complete_output_stream.as_bytes();
        self.current_stream_byte_position = AbsoluteByteIndex(output_bytes.len());
    }
}

impl std::ops::AddAssign<RelativeByteIndex> for AbsoluteByteIndex {
    fn add_assign(&mut self, relative: RelativeByteIndex) {
        self.0 += relative.0;
    }
}

fn convert_bytes_to_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("invalid UTF-8 in output stream")
}

/// Returns all named capture groups from regexes as variables.
fn process_captures(
    regex: &Regex,
    matched_text: &str)
    -> HashMap<String, String> {
    // We shouldn't be calling this function if it didn't match.
    debug_assert_eq!(regex.is_match(matched_text), true);

    let captures = if let Some(captures) = regex.captures(matched_text) {
        captures
    } else {
        return HashMap::new();
    };

    let mut variables = HashMap::new();

    for capture_name in regex.capture_names() {
        // we only care about named captures.
        if let Some(name) = capture_name {
            let captured_value = captures.name(name).unwrap();

            variables.insert(name.to_owned(), captured_value.as_str().to_owned());
        }
    }

    variables
}

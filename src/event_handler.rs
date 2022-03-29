//! Logic for showing testing events to the user - the UI logic.
//!
//! All "UI" logic is driven through the `EventHandler` trait.

pub use self::default::EventHandler as Default;

pub use crate::{Config, model::{TestResult}};

mod default;

/// An object which listens to events that occur during a test suite run.
pub trait EventHandler {
    /// Called to notify before the test suite has started.
    fn on_test_suite_started(&mut self, suite_details: &TestSuiteDetails, config: &Config);

    /// Called to notify when the entire test suite has finished execution.
    fn on_test_suite_finished(&mut self, passed: bool, config: &Config);

    /// Called to notify when a test has been executed.
    fn on_test_finished(&mut self, result: TestResult, config: &Config);

    /// Called to notify about a nonfatal warning.
    fn note_warning(&mut self, message: &str);
}

/// Stores details about the test suite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestSuiteDetails {
    /// The number of test files in the suite.
    pub number_of_test_files: usize,
}


//! Tests for the test evaluator state logic.

use crate::{
    Config,
    model::{self, TestFailReason},
};
use super::*;

const EMOJI_SMILEY: char = '\u{1F600}';
const EMOJI_JOY: char = '\u{1F602}';

fn fixture_program_prints_whitespace_emoji_and_hello_world() -> TestRunState {
    let mut test_state = TestRunState::new(HashMap::new());
    test_state.append_program_output(&format!("  \n{}\nhello \nworld", EMOJI_SMILEY));
    test_state
}

// Stress-test for byte<->char conversion logic.
fn fixture_program_prints_unicode_emoji() -> TestRunState {
    let mut test_state = TestRunState::new(HashMap::new());
    test_state.append_program_output(&format!("  {}\n  {} smiles.\n\t{}\njoy{}.", EMOJI_SMILEY, EMOJI_SMILEY, EMOJI_JOY, EMOJI_SMILEY));
    test_state
}

// Prints the periodic table in order, useful of testing line constraints.
fn fixture_program_prints_periodic_table_in_order() -> TestRunState {
    const ELEMENTS: &'static [&'static str] = &[
        "Hydrogen", "Helium", "Lithium", "Beryllium", "Boron", "Carbon",
        "Nitrogen", "Oxygen", "Fluorine", "Neon", "Sodium", "Magnesium",
    ];

    let mut test_state = TestRunState::new(HashMap::new());
    test_state.append_program_output(&ELEMENTS.join(", is an element.\n"));
    test_state
}

#[test]
fn check_next_works_standalone_in_very_basic_scenario() {
    let mut test_state = fixture_program_prints_whitespace_emoji_and_hello_world();
    let config = Config::default();

    assert!(test_state.unprocessed_output_stream().starts_with("  "));

    test_state.check_next(&model::PatternComponent::Text(EMOJI_SMILEY.to_string()).into(), &config).unwrap();
    assert_eq!(test_state.unprocessed_output_stream(), "hello \nworld");

    let res = test_state.check_next(&model::PatternComponent::Text("world".to_owned()).into(), &config);
    match res {
        TestResultKind::Fail { reason, hint } => {
            match reason {
                TestFailReason::CheckFailed(..) => {
                    assert_eq!(test_state.unprocessed_output_stream(), "hello \nworld",
                               "errors should not consume any of the underlying stream");
                    assert_eq!(hint, Some("found a match for \'world\', but it does not appear on the next line, as required by the CHECK-NEXT directive".to_owned()));
                },
                r => panic!("unexpected test failure reason: {:?}", r),
            }
        },
        _ => panic!("unexpected failure reason: {:?}", res),
    }

    test_state.check_next(&model::PatternComponent::Text("hello".to_owned()).into(), &config).unwrap();
    assert_eq!(test_state.unprocessed_output_stream(), "world");
}

#[test]
fn check_next_can_handle_multibyte_unicode_chars() {
    let mut test_state = fixture_program_prints_unicode_emoji();
    let config = Config::default();

    assert!(test_state.unprocessed_output_stream().starts_with("  "));

    // Consume first smiley emoji
    test_state.check_next(&model::PatternComponent::Text(EMOJI_SMILEY.to_string()).into(), &config).unwrap();
    assert!(test_state.unprocessed_output_stream().starts_with(&format!("  {} smiles.\n", EMOJI_SMILEY)));

    // Consume next identical smiley.
    test_state.check_next(&model::PatternComponent::Text(EMOJI_SMILEY.to_string()).into(), &config).unwrap();
    assert!(test_state.unprocessed_output_stream().starts_with("\t"));

    // Consume the joy emoji.
    test_state.check_next(&model::PatternComponent::Text(EMOJI_JOY.to_string()).into(), &config).unwrap();
    assert_eq!(test_state.unprocessed_output_stream(), format!("joy{}.", EMOJI_SMILEY));

    // Consume the last smiley and terminating full stop.
    test_state.check_next(&model::PatternComponent::Text(format!("{}.", EMOJI_SMILEY)).into(), &config).unwrap();
    assert_eq!(test_state.unprocessed_output_stream(), "");
}

#[test]
fn check_next_rejects_matches_not_on_next_line() {
    let mut test_state = fixture_program_prints_periodic_table_in_order();
    let config = Config::default();

    assert!(test_state.unprocessed_output_stream().starts_with("Hydrogen, is an element.\nHelium, is an element.\n"));

    test_state.check_next(&model::PatternComponent::Text("Hydrogen".to_owned()).into(), &config).unwrap();
    assert!(test_state.unprocessed_output_stream().starts_with("Helium"));

    // Attempt to read ahead of next line, expect failure.
    let res = test_state.check_next(&model::PatternComponent::Text("Lithium".to_owned()).into(), &config);
    match res {
        TestResultKind::Fail { reason, hint } => {
            match reason {
                TestFailReason::CheckFailed(..) => {
                    assert!(test_state.unprocessed_output_stream().starts_with("Helium"),
                            "errors should not consume any of the underlying stream");
                    assert_eq!(hint, Some("found a match for \'Lithium\', but it does not appear on the next line, as required by the CHECK-NEXT directive".to_owned()));
                },
                r => panic!("unexpected test failure reason: {:?}", r),
            }
        },
        _ => panic!("unexpected failure reason: {:?}", res),
    }
}

#[test]
fn check_with_nonexistent_regex_produces_failure() {
    let mut test_state = fixture_program_prints_periodic_table_in_order();
    let config = Config::default();

    assert!(test_state.unprocessed_output_stream().starts_with("Hydrogen, is an element.\nHelium, is an element.\n"));

    test_state.check(&model::PatternComponent::Text("Helium".to_owned()).into(), &config).unwrap();

    let res = test_state.check(&model::PatternComponent::Text("nonexistent".to_owned()).into(), &config);

    // Validate that a nonexistent regex triggers a failure.
    if let TestResultKind::Fail { reason, hint } = res {
        match reason {
            TestFailReason::CheckFailed(failure_info) => {
                assert!(failure_info.successfully_checked_text().ends_with("Helium, is an element.\n"));
                assert!(failure_info.remaining_text().starts_with("Lithium, is an element.\n"));
                assert_eq!(hint, None);
            },
            r => panic!("unexpected failure reason: {:?}", r),
        }
    } else {
        panic!("expected the pattern to fail: {:?}", res);
    }
}

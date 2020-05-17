//! Utility functions for internal use.

const DEFAULT_INDENT_ATOM: &'static str = "  ";
const TRUNCATED_TEXT_MARKER: &'static str = "... (truncated)";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TruncateDirection { Top, Bottom }

/// Indents a piece of text.
pub fn indent(text: &str, level: usize) -> String {
    indent_ext(text, level, DEFAULT_INDENT_ATOM)
}

pub fn indent_ext(text: &str, level: usize, indentation_atom: &str) -> String {
    let indent = (0..level).into_iter().map(|_| indentation_atom).collect::<Vec<_>>().join("");
    text.lines().map(|l| format!("{}{}", indent, l.trim())).collect::<Vec<_>>().join("\n") + "\n"
}

pub fn decorate_with_line_numbers(text: &str, starts_from_line_number: usize) -> String {
    let max_line_num_digits = (starts_from_line_number + text.lines().count()).to_string().len();

    text.lines().enumerate().map(|(relative_lineno, line)| {
        let line_number_str = (starts_from_line_number + relative_lineno).to_string();
        let number_of_pad_chars = max_line_num_digits - line_number_str.len();
        let horizontal_padding_str = (0..number_of_pad_chars).into_iter().map(|_| " ").collect::<String>();

        format!("{}{}|      {}", line_number_str, horizontal_padding_str, line)
    }).collect::<Vec<_>>().join("\n")
}

pub fn truncate_to_max_lines(
    text: &str,
    max_line_count: usize,
    truncate_direction: TruncateDirection) -> String {
    let lines = text.lines().collect::<Vec<_>>();

    let is_truncated = lines.len() > max_line_count;

    let truncated_lines: Vec<_> = match truncate_direction {
        TruncateDirection::Bottom => lines.into_iter().take(max_line_count).collect(),
        TruncateDirection::Top => lines.into_iter().rev().take(max_line_count).rev().collect(),
    };

    let truncated_text = truncated_lines.join("\n");

    if is_truncated {
        match truncate_direction {
            TruncateDirection::Bottom => truncated_text.to_owned() + "\n\n" + TRUNCATED_TEXT_MARKER,
            TruncateDirection::Top => TRUNCATED_TEXT_MARKER.to_string() + "\n\n" + &truncated_text[..],
        }
    } else {
        truncated_text // the text was not actually truncated
    }
}

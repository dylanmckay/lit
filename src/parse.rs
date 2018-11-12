use model::*;

use regex::Regex;
use std::mem;
use std::path::Path;

lazy_static! {
    static ref DIRECTIVE_REGEX: Regex = Regex::new("([A-Z-]+):(.*)").unwrap();
    static ref IDENTIFIER_REGEX: Regex = Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
}

/// Parses a test file
pub fn test_file<P,I>(path: P, chars: I) -> Result<Test, String>
    where P: AsRef<Path>, I: Iterator<Item=char> {
    let mut directives = Vec::new();
    let test_body: String = chars.collect();

    let path = path.as_ref().to_owned();

    for (line_idx, line) in test_body.lines().enumerate() {
        let line_number = line_idx + 1;

        match self::possible_directive(line, line_number as _) {
            Some(Ok(directive)) => directives.push(directive),
            Some(Err(e)) => {
                return Err(format!(
                    "could not parse directive: {}", e)
                );
            },
            None => continue,
        }
    }

    Ok(Test {
        path,
        directives: directives,
    })
}


/// Parses a tool invocation.
///
/// It is generatlly in the format:
///
/// ``` bash
/// <tool-name> [arg1] [arg2] ...
/// ```
pub fn invocation<'a,I>(words: I) -> Result<Invocation, String>
    where I: Iterator<Item=&'a str> {
    let parts: Vec<_> = words.collect();
    let original_command = parts.join(" ");

    Ok(Invocation { original_command })
}

pub fn text_pattern(s: &str) -> TextPattern {
    let mut components: Vec<PatternComponent> = vec![];
    let mut chars = s.chars().peekable();

    let mut current_text = vec![];

    loop {
        let complete_text = |current_text: &mut Vec<char>, components: &mut Vec<PatternComponent>| {
            let text = mem::replace(current_text, Vec::new())
                .into_iter().collect();
            components.push(PatternComponent::Text(text));
        };

        match (chars.next(), chars.peek().cloned()) {
            // Variable.
            (Some('$'), Some('$')) => {
                complete_text(&mut current_text, &mut components);
                chars.next(); // Eat second '$'.

                let name: String = chars.clone()
                                        .take_while(|c| c.is_alphanumeric())
                                        .collect();
                chars.nth(name.len() - 1); // Skip the variable name.
                components.push(PatternComponent::Variable(name));
            },
            // Named or unnamed regex.
            (Some('['), Some('[')) => {
                complete_text(&mut current_text, &mut components);
                chars.next(); // Eat second `[`

                let mut current_regex = vec![];
                let mut bracket_level = 0i32;
                loop {
                    match (chars.next(), chars.peek().cloned()) {
                        (Some(']'), Some(']')) if bracket_level == 0=> {
                            chars.next(); // Eat second `]`.
                            break;
                        },
                        (Some(c), _) => {
                            match c {
                                '[' => bracket_level += 1,
                                ']' => bracket_level -= 1,
                                _ => (),
                            }

                            current_regex.push(c);
                        },
                        (None, _) => {
                            break;
                        },
                    }
                }

                let regex: String = current_regex.into_iter().collect();

                let first_colon_idx = regex.chars().position(|c| c == ':');
                let (name, regex): (Option<&str>, &str) = match first_colon_idx {
                    Some(first_colon_idx) => {
                        let substr = &regex[0..first_colon_idx];

                        if IDENTIFIER_REGEX.is_match(&substr) {
                            (Some(substr), &regex[first_colon_idx+1..])
                        } else {
                            (None, &regex)
                        }
                    },
                    None => (None, &regex),
                };

                match name {
                    Some(name) => components.push(PatternComponent::NamedRegex { name: name.to_owned(), regex: regex.to_owned() }),
                    None => components.push(PatternComponent::Regex(regex.to_owned())),
                }

            },
            (Some(c), _) => {
                current_text.push(c);
            },
            (None, _) => {
                complete_text(&mut current_text, &mut components);
                break;
            }
        }
    }

    TextPattern { components: components }
}

/// Parses a possible directive, if a string defines one.
///
/// Returns `None` if no directive is specified.
pub fn possible_directive(string: &str, line: u32)
    -> Option<Result<Directive, String>> {
    if !DIRECTIVE_REGEX.is_match(string) { return None; }

    let captures = DIRECTIVE_REGEX.captures(string).unwrap();
    let command_str = captures.get(1).unwrap().as_str().trim();
    let after_command_str = captures.get(2).unwrap().as_str().trim();

    match command_str {
        // FIXME: better message if we have 'RUN :'
        "RUN" => {
            let inner_words = after_command_str.split_whitespace();
            let invocation = match self::invocation(inner_words) {
                Ok(i) => i,
                Err(e) => return Some(Err(e)),
            };

            Some(Ok(Directive::new(Command::Run(invocation), line)))
        },
        "CHECK" => {
            let text_pattern = self::text_pattern(after_command_str);
            Some(Ok(Directive::new(Command::Check(text_pattern), line)))
        },
        "CHECK-NEXT" => {
            let text_pattern = self::text_pattern(after_command_str);
            Some(Ok(Directive::new(Command::CheckNext(text_pattern), line)))
        },
        "XFAIL" => {
            Some(Ok(Directive::new(Command::XFail, line)))
        },
        _ => {
            Some(Err(format!("command '{}' not known", command_str)))
        },
    }
}

#[cfg(tes)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn parses_single_text() {
        assert_eq!(text_pattern("hello world"),
                   "hello world");
    }

    #[test]
    fn correctly_escapes_text() {
        assert_eq!(text_pattern("hello()").as_str(),
                   "hello\\(\\)");
    }

    #[test]
    fn correctly_picks_up_single_regex() {
        assert_eq!(text_pattern("[[\\d]]").as_str(),
                   "\\d");
    }

    #[test]
    fn correctly_picks_up_regex_between_text() {
        assert_eq!(text_pattern("1[[\\d]]3").as_str(),
                   "1\\d3");
    }

    #[test]
    fn correctly_picks_up_named_regex() {
        assert_eq!(text_pattern("[[num:\\d]]").as_str(),
                   "(?P<num>\\d)");
    }
}


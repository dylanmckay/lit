//! Utilities for resolving/substituting variables within different types.

use crate::model::*;
use crate::vars::Variables;
use crate::Config;

use regex::Regex;

lazy_static! {
    static ref CONSTANT_REGEX: Regex = Regex::new("@([_a-zA-Z]+)").unwrap();
}

/// A span representing where a constant name resides in a string.
#[derive(Debug)]
struct ConstantSpan {
    /// The name of the constant.
    name: String,
    /// The index of the first character.
    start: usize,
    /// The index of the last character.
    end: usize,
}

pub fn text_pattern(pattern: &TextPattern, config: &Config,
                    variables: &mut Variables) -> Regex {
    let regex_parts: Vec<_> = pattern.components.iter().map(|comp| match *comp {
        PatternComponent::Text(ref text) => regex::escape(text),
        PatternComponent::Constant(ref name) | PatternComponent::Variable(ref name) => {
            // FIXME: proper error handling.
            let value = config.lookup_variable(name, variables);

            let var_resolution_log = format!("resolving '@{}' to '{}' in {:?}", name, value, pattern);
            debug!("{}", var_resolution_log);

            if config.dump_variable_resolution {
                eprintln!("[info] {}", var_resolution_log);
            }

            value.to_owned()
        },
        PatternComponent::Regex(ref regex) => regex.clone(),
        PatternComponent::NamedRegex { ref name, ref regex } => format!("(?P<{}>{})", name, regex),
    }).collect();
    Regex::new(&regex_parts.join("")).expect("generated invalid line match regex")
}

pub fn invocation(invocation: &Invocation,
                  config: &Config,
                  constants: &mut Variables) -> String {
    let mut command_line = String::new();

    let _cmd: String = invocation.original_command.clone();
    let mut constant_spans = CONSTANT_REGEX.find_iter(&_cmd).map(|mat| {
        let name = mat.as_str()[1..].to_owned(); // Skip the '@' character.

        ConstantSpan {
            name: name,
            start: mat.start(),
            end: mat.end(),
        }
    });

    let mut index = 0;
    loop {
        if let Some(next_span) = constant_spans.next() {
            assert!(index <= next_span.start, "went too far");

            let value = config.lookup_variable(&next_span.name, constants);

            let var_resolution_log = format!("resolving '@{}' to '{}' in {:?}", next_span.name, value, _cmd);
            debug!("{}", var_resolution_log);

            if config.dump_variable_resolution {
                eprintln!("[info] {}", var_resolution_log);
            }

            // Check if there is some text between us and the regex.
            if next_span.start != index {
                let part = &invocation.original_command[index..next_span.start];

                command_line += part;
                index += part.len();
            }

            assert_eq!(index, next_span.start, "we should be up to the regex");
            command_line += &value;
            index += next_span.name.len() + 1; // Skip the `@` and the name.
        } else {
            // Almost finished, just copy over the rest of the text.
            command_line += &invocation.original_command[index..];
            break;
        }
    }

    command_line
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    lazy_static! {
        static ref VARIABLES: HashMap<String, String> = {
            let mut v = HashMap::new();
            v.insert("po".to_owned(), "polonium".to_owned());
            v.insert("name".to_owned(), "bob".to_owned());
            v
        };
    }

    mod text_pattern {
        use super::*;
        use crate::{parse, vars};
        use crate::Config;

        fn resolve(s: &str) -> String {
            let text_pattern = parse::text_pattern(s);
            vars::resolve::text_pattern(&text_pattern, &Config::default(), &mut VARIABLES.clone()).as_str().to_owned()
        }

        #[test]
        fn correctly_picks_up_single_variable() {
            assert_eq!(resolve("$$po").as_str(),
                       "polonium");
        }

        #[test]
        fn correctly_picks_up_variable_between_junk() {
            assert_eq!(resolve("[[[a-z]]]$$po foo").as_str(),
                       "[a-z]polonium foo");
        }

        #[test]
        fn correctly_picks_up_variable_at_end() {
            assert_eq!(resolve("goodbye $$name").as_str(),
                       "goodbye bob");
        }
    }

    mod invocation {
        use crate::{parse, vars, Config};
        use std::collections::HashMap;

        lazy_static! {
            static ref BASIC_CONSTANTS: HashMap<String, String> = {
                let mut m = HashMap::new();
                m.insert("cc".to_owned(), "clang++".to_owned());
                m
            };
        }

        fn resolve(s: &str, consts: &mut HashMap<String, String>) -> String {
            let invocation = parse::invocation(s.split_whitespace()).unwrap();
            vars::resolve::invocation(&invocation, &Config::default(), consts)
        }

        #[test]
        fn no_constants_is_nop() {
            assert_eq!(resolve("hello world", &mut BASIC_CONSTANTS.clone()), "hello world");
        }

        #[test]
        fn only_const() {
            assert_eq!(resolve("@cc", &mut BASIC_CONSTANTS.clone()), "clang++");
        }

        #[test]
        fn junk_then_const() {
            assert_eq!(resolve("foo bar! @cc", &mut BASIC_CONSTANTS.clone()), "foo bar! clang++");
        }

        #[test]
        fn junk_then_const_then_junk() {
            assert_eq!(resolve("hello @cc world", &mut BASIC_CONSTANTS.clone()), "hello clang++ world");
        }
    }
}


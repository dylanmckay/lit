// FIXME: Rename this to `CommandLine`.

use Config;
use regex::Regex;
use std::collections::HashMap;

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

impl Invocation
{
    pub fn resolve(&self, config: &Config, constants: &mut HashMap<String, String>) -> String {
        let mut command_line = String::new();

        let _cmd: String = self.original_command.clone();
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

                let constant_value = config.lookup_variable(&next_span.name, constants);

                // Check if there is some text between us and the regex.
                if next_span.start != index {
                    let part = &self.original_command[index..next_span.start];

                    command_line += part;
                    index += part.len();
                }

                assert_eq!(index, next_span.start, "we should be up to the regex");
                command_line += &constant_value;
                index += next_span.name.len() + 1; // Skip the `@` and the name.
            } else {
                // Almost finished, just copy over the rest of the text.
                command_line += &self.original_command[index..];
                break;
            }
        }

        command_line
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    lazy_static! {
        static ref BASIC_CONSTANTS: HashMap<String, String> = {
            let mut m = HashMap::new();
            m.insert("cc".to_owned(), "clang++".to_owned());
            m
        };
    }

    fn resolve(s: &str, consts: &mut HashMap<String, String>) -> String {
        let invocation = Invocation::parse(s.split_whitespace()).unwrap();
        invocation.resolve(&Config::default(), consts)
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

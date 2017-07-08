// FIXME: Rename this to `CommandLine`.

use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref CONSTANT_REGEX: Regex = Regex::new("@([a-z]+)").unwrap();
}

/// A tool invocation.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Invocation
{
    original_command: String,
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
    /// Parses a tool invocation.
    ///
    /// It is generatlly in the format:
    ///
    /// ``` bash
    /// <tool-name> [arg1] [arg2] ...
    /// ```
    pub fn parse<'a,I>(words: I) -> Result<Self,String>
        where I: Iterator<Item=&'a str> {
        let parts: Vec<_> = words.collect();
        let original_command = parts.join(" ");

        Ok(Invocation {
            original_command: original_command,
        })
    }

    pub fn resolve(&self, constants: &HashMap<String, String>) -> String {
        let mut command_line = String::new();

        let _cmd = self.original_command.clone();
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

                let constant_value = constants.get(&next_span.name)
                    .expect("no constant with that name exists");

                // Check if there is some text between us and the regex.
                if next_span.start != index {
                    let part = &self.original_command[index..next_span.start];

                    command_line += part;
                    index += part.len();
                }

                assert_eq!(index, next_span.start, "we should be up to the regex");
                command_line += constant_value;
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

    fn resolve(s: &str, consts: &HashMap<String, String>) -> String {
        let invocation = Invocation::parse(s.split_whitespace()).unwrap();
        invocation.resolve(consts)
    }

    #[test]
    fn no_constants_is_nop() {
        assert_eq!(resolve("hello world", &BASIC_CONSTANTS), "hello world");
    }

    #[test]
    fn only_const() {
        assert_eq!(resolve("@cc", &BASIC_CONSTANTS), "clang++");
    }

    #[test]
    fn junk_then_const() {
        assert_eq!(resolve("foo bar! @cc", &BASIC_CONSTANTS), "foo bar! clang++");
    }

    #[test]
    fn junk_then_const_then_junk() {
        assert_eq!(resolve("hello @cc world", &BASIC_CONSTANTS), "hello clang++ world");
    }
}

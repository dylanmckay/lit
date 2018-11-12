use Config;
use regex::{self, Regex};
use model::*;

use std::collections::HashMap;

impl Matcher {
    pub fn resolve(&self, config: &Config,
                   variables: &mut HashMap<String, String>) -> Regex {
        let regex_parts: Vec<_> = self.components.iter().map(|comp| match *comp {
            Component::Text(ref text) => regex::escape(text),
            Component::Variable(ref name) => {
                // FIXME: proper error handling.
                let value = config.lookup_variable(name, variables);
                value.to_owned()
            },
            Component::Regex(ref regex) => regex.clone(),
            Component::NamedRegex { ref name, ref regex } => format!("(?P<{}>{})", name, regex),
        }).collect();
        Regex::new(&regex_parts.join("")).expect("generated invalid line match regex")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    lazy_static! {
        static ref VARIABLES: HashMap<String, String> = {
            let mut v = HashMap::new();
            v.insert("po".to_owned(), "polonium".to_owned());
            v.insert("name".to_owned(), "bob".to_owned());
            v
        };
    }

    fn matcher(s: &str) -> String {
        Matcher::parse(s).resolve(&Config::default(), &mut VARIABLES.clone()).as_str().to_owned()
    }

    #[test]
    fn parses_single_text() {
        assert_eq!(matcher("hello world"),
                   "hello world");
    }

    #[test]
    fn correctly_escapes_text() {
        assert_eq!(matcher("hello()").as_str(),
                   "hello\\(\\)");
    }

    #[test]
    fn correctly_picks_up_single_regex() {
        assert_eq!(matcher("[[\\d]]").as_str(),
                   "\\d");
    }

    #[test]
    fn correctly_picks_up_regex_between_text() {
        assert_eq!(matcher("1[[\\d]]3").as_str(),
                   "1\\d3");
    }

    #[test]
    fn correctly_picks_up_named_regex() {
        assert_eq!(matcher("[[num:\\d]]").as_str(),
                   "(?P<num>\\d)");
    }

    #[test]
    fn correctly_picks_up_single_variable() {
        assert_eq!(matcher("$$po").as_str(),
                   "polonium");
    }

    #[test]
    fn correctly_picks_up_variable_between_junk() {
        assert_eq!(matcher("[[[a-z]]]$$po foo").as_str(),
                   "[a-z]polonium foo");
    }

    #[test]
    fn correctly_picks_up_variable_at_end() {
        assert_eq!(matcher("goodbye $$name").as_str(),
                   "goodbye bob");
    }
}

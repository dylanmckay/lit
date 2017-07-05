use {Test, Config};

/// A constant.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Constant
{
    /// The path of the test that is being run.
    TestPath,
    /// A custom constant
    /// /// A custom constant.
    Custom { value: String },
}

impl Constant
{
    /// Maps a constant name to a constant.
    /// Returns `None` if no mapping exists.
    pub fn lookup(name: &str, config: &Config) -> Option<Constant> {
        match name {
            "file" => Some(Constant::TestPath),
            _ => {
                config.constants.get(name).map(|value| Constant::Custom { value: value.clone() })
            },
        }
    }
}

/// An argument to a tool.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Argument
{
    Normal(String),
    Substitute(Constant),
}

impl Argument
{
    /// Parses an argument to a tool.
    ///
    /// If it is prefixed with `@`, then it will be taken
    /// as a constant substitution, otherwise it will
    /// be read verbatim as a tool argument.
    pub fn parse(string: String, config: &Config) -> Result<Self,String> {
        // check if we are parsing a substitution
        if string.chars().next().unwrap() == '@' {
            let name: String = string.chars().skip(1).collect();

            match Constant::lookup(&name, config) {
                Some(constant) => Ok(Argument::Substitute(constant)),
                None => Err(format!("constant does not exist: {}", name)),
            }
        } else { // it is a plain old argument
            Ok(Argument::Normal(string))
        }
    }

    pub fn resolve(&self, test: &Test) -> String {
        match *self {
            Argument::Normal(ref s) => s.clone(),
            Argument::Substitute(ref constant) => match *constant {
                Constant::TestPath => test.path.clone(),
                Constant::Custom { ref value } => value.clone(),
            },
        }
    }
}

/// A tool invocation.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Invocation
{
    pub executable: String,
    pub arguments: Vec<Argument>,
}

impl Invocation
{
    /// Parses a tool invocation.
    ///
    /// It is in the format:
    ///
    /// ``` bash
    /// <tool-name> [arg1] [arg2] ...
    /// ```
    pub fn parse<'a,I>(mut words: I, config: &Config) -> Result<Self,String>
        where I: Iterator<Item=&'a str> {
        let executable = match words.next() {
            Some(exec) => exec,
            None => return Err("no executable specified".into()),
        }.into();

        let mut arguments = Vec::new();

        for arg_str in words {
            let arg = Argument::parse(arg_str.into(), config)?;
            arguments.push(arg);
        }

        Ok(Invocation {
            executable: executable,
            arguments: arguments,
        })
    }
}

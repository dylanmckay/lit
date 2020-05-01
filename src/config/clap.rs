//! Routines for exposing a command line interface via the `clap` crate.
//!
//! These routines can be used to update `Config` objects with automatic CLI arguments.

use crate::Config;
use clap::{App, Arg, ArgMatches};

/// The set of available debug parameters.
const DEBUG_OPTION_VALUES: &'static [(&'static str, fn(&mut Config))] = &[
    ("variable-resolution", |config: &mut Config| {
        config.dump_variable_resolution = true;
    }),
];

lazy_static! {
    static ref DEBUG_OPTION_HELP: String = {
        let debug_option_vals = DEBUG_OPTION_VALUES.iter().map(|d| d.0).collect::<Vec<_>>();
        let debug_option_vals = debug_option_vals.join(", ");

        format!("Enabled debug output. Possible debugging flags are: {}.", debug_option_vals)
    };
}


/// Mounts extra arguments that can be used to fine-tune testing
/// into a `clap` CLI application.
pub fn mount_inside_app<'a, 'b>(
    app: App<'a, 'b>,
    test_paths_as_positional_arguments: bool,
) -> App<'a, 'b> {
    let app = app
        .arg(Arg::with_name("supported-file-extension")
            .long("add-file-extension")
            .takes_value(true)
            .value_name("EXT")
            .multiple(true)
            .help("Adds a file extension to the test search list. Extensions can be specified either with or without a leading period"))
        .arg(Arg::with_name("constant")
            .long("define-constant")
            .short("c")
            .takes_value(true)
            .value_name("NAME>=<VALUE") // this shows as '<NAME>=<VALUE>'
            .multiple(true)
            .help("Sets a constant, accessible in the test via '@<NAME>"))
        .arg(Arg::with_name("keep-tempfiles")
            .long("keep-tempfiles")
            .help("Disables automatic deletion of tempfiles generated during the test run"))
        .arg(Arg::with_name("debug-all")
            .long("debug-all")
            .short("g")
            .help("Turn on all debugging flags"))
        .arg(Arg::with_name("debug")
            .long("debug")
            .takes_value(true)
            .value_name("FLAG")
            .multiple(true)
            .help(&DEBUG_OPTION_HELP[..]));

    // Test paths argument
    let test_paths_arg = {
        let mut arg = Arg::with_name("add-tests")
            // .long("add-tests")
            .takes_value(true)
            .value_name("PATH TO TEST OR TESTS")
            .multiple(true)
            .help("Adds a path to the test search pathset. If the path refers to a directory, it will be recursed, if it refers to a file, it will be treated as a test file");

        // If positional arguments are disabled, add this as a longhand option anyway.
        if !test_paths_as_positional_arguments {
            arg = arg.long("add-tests");
        }

        arg
    };

    let app = app
        .arg(test_paths_arg);

    app
}

/// Parses command line arguments from `clap` into a destination `Config` object.
pub fn parse_arguments(matches: &ArgMatches,
                       destination_config: &mut Config) {
    if let Some(extensions) = matches.values_of("supported-file-extension") {
        for extension in extensions {
            destination_config.add_extension(extension);
        }
    }

    if let Some(test_paths) = matches.values_of("add-tests") {
        for test_path in test_paths {
            destination_config.add_search_path(test_path);
        }
    }

    if let Some(constant_define_strs) = matches.values_of("constant") {
        for constant_define_str in constant_define_strs {
            let constant_definition: ConstantDefinition = match constant_define_str.parse() {
                Ok(c) => c,
                Err(e) => panic!(e),
            };

            destination_config.constants.insert(constant_definition.name, constant_definition.value);
        }
    }

    if matches.is_present("keep-tempfiles") {
        destination_config.cleanup_temporary_files = false;
    }

    if let Some(debug_flags) = matches.values_of("debug") {
        for debug_flag in debug_flags {
            let apply_fn = DEBUG_OPTION_VALUES.iter().find(|(k, _)| k == &debug_flag.trim()).map(|d| d.1);

            match apply_fn {
                Some(func) => func(destination_config),
                None => panic!("no debugging flag named '{}'", debug_flag),
            }
        }
    }

    if matches.is_present("debug-all") {
        for (_, debug_flag_fn) in DEBUG_OPTION_VALUES {
            debug_flag_fn(destination_config);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConstantDefinition {
    pub name: String,
    pub value: String,
}

impl std::str::FromStr for ConstantDefinition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        if s.chars().filter(|&c| c == '=').count() != 1 {
            return Err(format!("constant definition must have exactly one equals sign but got '{}", s))
        }
        if s.len() < 3 {
            return Err(format!("constant definitions must include both a <NAME> and a <VALUE>, separated by equals"));
        }

        let (name, value) = s.split_at(s.find('=').unwrap());
        let value = &value[1..]; // trim equals
        let (name, value) = (name.trim().to_owned(), value.trim().to_owned());

        Ok(ConstantDefinition { name, value })
    }
}

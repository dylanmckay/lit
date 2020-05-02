//! Routines for exposing a command line interface via the `clap` crate.
//!
//! These routines can be used to update `Config` objects with automatic CLI arguments.

use crate::Config;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::io::Write;

/// The set of available debug parameters.
const DEBUG_OPTION_VALUES: &'static [(&'static str, fn(&mut Config))] = &[
    ("variable-resolution", |config: &mut Config| {
        config.dump_variable_resolution = true;
    }),
];

const SHOW_OPTION_VALUES: &'static [(&'static str, fn(&Config, &mut dyn Write) -> std::io::Result<()>)] = &[
    ("test-file-paths", |config, writer| {
        let test_file_paths = crate::run::find_files::with_config(config).unwrap();
        for test_file_path in test_file_paths {
            writeln!(writer, "{}", test_file_path)?;
        }

        Ok(())

    }),
    ("lit-config", |config, writer| {
        writeln!(writer, "{:#?}", config)
    }),
];

lazy_static! {
    static ref DEBUG_OPTION_HELP: String = {
        let debug_option_vals = DEBUG_OPTION_VALUES.iter().map(|d| d.0).collect::<Vec<_>>();
        let debug_option_vals = debug_option_vals.join(", ");

        format!("Enable debug output. Possible debugging flags are: {}.", debug_option_vals)
    };

    static ref SHOW_SUBCOMMAND_WHAT_OPTION_HELP: String = {
        let show_option_vals = SHOW_OPTION_VALUES.iter().map(|d| format!("    - {}", d.0)).collect::<Vec<_>>();
        let show_option_vals = show_option_vals.join("\n");

        format!("Show only a specific value. Possible values are:\n{}\nIf this value is not specified, all values are shown", show_option_vals)
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
            .help(&DEBUG_OPTION_HELP[..]))
        .subcommand(SubCommand::with_name("show")
            .about("Shows information about the test suite, without running tests")
            .arg(Arg::with_name("what")
                .takes_value(true)
                .value_name("WHAT")
                .help(&SHOW_SUBCOMMAND_WHAT_OPTION_HELP)));

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

    // NOTE: should process subcommands at the very end
    if let Some(matches) = matches.subcommand_matches("show") {
        let what_fns: Vec<_> = match matches.value_of("what") {
            Some(what) => {
                match SHOW_OPTION_VALUES.iter().find(|(name, _)| *name == what) {
                    Some((name, what_fn)) => vec![(name, what_fn)],
                    None => {
                        eprintln!("error: unknown show value: '{}'", what);
                        std::process::exit(1);
                    },
                }
            },
            None => {
                SHOW_OPTION_VALUES.iter().map(|(name, f)| (name, f)).collect()
            },
        };

        let writer = &mut std::io::stdout();

        let show_labels = what_fns.len() > 1;
        for (label, what_fn) in what_fns {
            if show_labels {
                writeln!(writer, "=================================================================").unwrap();
                writeln!(writer, "{}:", label).unwrap();
                writeln!(writer, "=================================================================").unwrap();
                writeln!(writer, "").unwrap();
            }

            what_fn(&destination_config, writer).unwrap();

            if show_labels {
                writeln!(writer, "").unwrap();
            }
        }

        // No tests should be ran when running this subcommand.
        std::process::exit(0);
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

//! Routines for exposing a command line interface via the `clap` crate.
//!
//! These routines can be used to update `Config` objects with automatic CLI arguments.

use crate::Config;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::{io::Write, path::Path};

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

const MULTIPLY_TRUNCATION_LINES_BY_THIS_AT_EACH_VERBOSITY_LEVEL: usize = 4;

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
        .arg(Arg::with_name("show-context-lines")
            .long("show-context-lines")
            .short("C")
            .takes_value(true)
            .value_name("NUMBER OF CONTEXT LINES")
            .help("Sets the number of output lines to be displayed when showing failure context. Set to '-1' to disable truncation."))
        .arg(Arg::with_name("always-show-stderr")
            .long("always-show-stderr")
            .help("Always echo the stderr streams emitted by programs under test. By default this is only done if the program exits with an error code. Stderr is also always printed when verbose mode is on."))
        .arg(Arg::with_name("keep-tempfiles")
            .long("keep-tempfiles")
            .help("Disables automatic deletion of tempfiles generated during the test run"))
        .arg(Arg::with_name("save-artifacts-to")
            .long("save-artifacts-to")
            .short("O")
            .takes_value(true)
            .value_name("DIRECTORY")
            .help("Exports all program outputs, temporary files, and logs, to a directory at the specified path. Will create the directory if it does not yet exist."))
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("Increase the level of verbosity in the output. Pass '-vv' for maximum verbosity"))
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

    if let Some(artifacts_path) = matches.value_of("save-artifacts-to") {
        destination_config.save_artifacts_to_directory = Some(Path::new(artifacts_path).to_owned());
    }

    // Parse verbosity.
    {
        let verbosity_level = matches.occurrences_of("verbose");

        if verbosity_level > 2 {
            warning(format!("the current verbosity level of '{}' specified is redundant, the maximum verbosity is '-vv' (corresponding to verbosity level 2)", verbosity_level));
        }

        if verbosity_level > 0 {
            if let Some(truncation) = destination_config.truncate_output_context_to_number_of_lines {
                destination_config.truncate_output_context_to_number_of_lines = Some(truncation * MULTIPLY_TRUNCATION_LINES_BY_THIS_AT_EACH_VERBOSITY_LEVEL * (verbosity_level as usize));
            }

            if verbosity_level >= 1 {
                destination_config.always_show_stderr = true;
            }

            if verbosity_level >= 2 {
                destination_config.dump_variable_resolution = true;
            }
        }
    }

    if matches.is_present("always-show-stderr") {
        destination_config.always_show_stderr = true;
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

    if let Some(cli_show_context_lines) = matches.value_of("show-context-lines") {
        match cli_show_context_lines.parse::<isize>() {
            Ok(-1) => {
                destination_config.truncate_output_context_to_number_of_lines = None;
            },
            Ok(lines) if lines < 0 => fatal_error(format!("invalid number of context lines: '{}' - must be a positive integer, or '-1' to disable truncation", cli_show_context_lines)),
            Ok(lines) => {
                destination_config.truncate_output_context_to_number_of_lines = Some(lines as usize);
            },
            Err(_) => fatal_error(format!("invalid number of context lines: '{}' - must be a positive integer, or '-1' to disable truncation", cli_show_context_lines)),
        }
    }

    // NOTE: should process subcommands at the very end
    if let Some(matches) = matches.subcommand_matches("show") {
        let what_fns: Vec<_> = match matches.value_of("what") {
            Some(what) => {
                match SHOW_OPTION_VALUES.iter().find(|(name, _)| *name == what) {
                    Some((name, what_fn)) => vec![(name, what_fn)],
                    None => {
                        fatal_error(format!("error: unknown show value: '{}'", what));
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

fn fatal_error(msg: impl AsRef<str>) -> ! {
    eprintln!("error: {}", msg.as_ref());
    std::process::exit(1);
}

fn warning(msg: impl AsRef<str>) {
    eprintln!("warning: {}", msg.as_ref());
}

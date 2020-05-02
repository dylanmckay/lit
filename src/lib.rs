//! A reusable testing tool, inspired by LLVM's `lit` tool.
//!
//! `lit` standing for _LLVM Integrated Test_.
//!
//! This crate contains both a reusable library for creating test tools and
//! an executable with generalized command line interface for manual usage.

pub use self::config::Config;

pub use self::errors::*;
pub use self::vars::{Variables, VariablesExt};

// The file extensions used by the integration tests for this repository.
#[doc(hidden)]
pub const INTEGRATION_TEST_FILE_EXTENSIONS: &'static [&'static str] = &[
    "txt", "sh",
];

pub mod config;
mod errors;
pub mod event_handler;
mod model;
mod parse;
pub mod run;
mod vars;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

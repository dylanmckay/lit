//! Functions for retrieving lists of files from disk.

use crate::{Config, model::TestFilePath};

use std;
use std::path::Path;
use walkdir::WalkDir;

/// Recursively finds tests for the given paths.
pub fn with_config(config: &Config) -> Result<Vec<TestFilePath>, String> {
    let mut absolute_paths = Vec::new();

    for path in config.test_paths.iter() {
        let path_str = path.display().to_string();

        let test_paths = in_path(&path_str, config)?;
        absolute_paths.extend(test_paths.into_iter().map(|p| Path::new(&p).to_owned()));
    }

    let test_paths = absolute_paths.into_iter().map(|absolute_path| {
        let relative_path =  // TODO: find most specific path in search tree. failing that, find most common path from all test paths and use that. otherwise use a random one like 'test'. maybe rename 'relative_path' to 'relative_path_for_display'.
        TestFilePath { absolute: absolute_path, relative: relative_path }
    }).collect();

    Ok(test_paths)
}

pub fn in_path(path: &str,
               config: &Config)
    -> Result<Vec<String>,String> {
    let metadata = match std::fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) => return Err(format!("failed to open '{}': {}",
                                     path, e)),
    };

    if metadata.is_dir() {
        tests_in_dir(path, config)
    } else {
        Ok(vec![path.to_owned()])
    }
}

fn tests_in_dir(path: &str,
                config: &Config) -> Result<Vec<String>,String> {
    let tests = files_in_dir(path)?.into_iter()
                     .filter(|f| {
                         let path = std::path::Path::new(f);
                         path.extension().map(|ext| config.is_extension_supported(ext.to_str().unwrap())).unwrap_or(false)
                     })
                     .collect();
    Ok(tests)
}

fn files_in_dir(path: &str) -> Result<Vec<String>,String> {
    let mut dir_tests = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();

        // don't go into an infinite loop
        if entry.path().to_str().unwrap() == path {
            continue;
        }

        if entry.metadata().unwrap().is_file() {
            dir_tests.push(entry.path().to_str().unwrap().to_owned());
        }
    }

    Ok(dir_tests)
}


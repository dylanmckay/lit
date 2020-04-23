//! Functions for retrieving lists of files from disk.

use Config;

use std;
use walkdir::WalkDir;

/// Recursively finds tests for the given paths.
pub fn with_config(config: &Config) -> Result<Vec<String>,String> {
    let mut tests = Vec::new();

    for path in config.test_paths.iter() {
        let path_str = path.display().to_string();
        let path_tests = in_path(&path_str, config)?;
        tests.extend(path_tests.into_iter());
    }

    Ok(tests)
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


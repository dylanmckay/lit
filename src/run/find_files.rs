//! Functions for retrieving lists of files from disk.

use crate::{Config, model::TestFilePath};

use std;
use std::path::{Path, PathBuf};
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
        let relative_path =  relative_path::compute(&absolute_path, config).expect("could not compute relative path");

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

mod relative_path {
    use crate::Config;
    use std::path::{Path, PathBuf};

    pub fn compute(test_absolute_path: &Path, config: &Config)
        -> Option<PathBuf> {
        let mut take_path_relative_to_dir = None;

        if take_path_relative_to_dir.is_none() {
            if let Some(least_specific_parent_test_search_directory_path) =
                least_specific_parent_test_search_directory_path(test_absolute_path, config) {
                take_path_relative_to_dir = Some(least_specific_parent_test_search_directory_path);
            }
        }

        if take_path_relative_to_dir.is_none() {
            if let Some(most_common_test_path_ancestor) =
                most_common_test_path_ancestor(test_absolute_path, config) {
                take_path_relative_to_dir = Some(most_common_test_path_ancestor);
            }
        }

        take_path_relative_to_dir.map(|relative_to| {
            test_absolute_path.strip_prefix(relative_to).expect("relative path computation failed: not a prefix").to_owned()
        })
    }

    /// Attempt to find the most specific prefix directory from the test search paths in the config.
    fn least_specific_parent_test_search_directory_path(test_absolute_path: &Path, config: &Config)
        -> Option<PathBuf> {
        // N.B. we iterate over the test paths here. We don't check for the directory's actual
        // existence on the filesystem. This makes testing easier, but also: test paths can only
        // be strict prefixes/supersets of other test paths if they ARE directories.
        let matching_parent_test_search_directories = config.test_paths.iter()
            .filter(|possible_dir_path| test_absolute_path.starts_with(possible_dir_path));

        let least_specific_matching_test_search_directory = matching_parent_test_search_directories.min_by_key(|p| p.components().count());

        if let Some(least_specific_matching_test_search_directory) = least_specific_matching_test_search_directory {
            Some(least_specific_matching_test_search_directory.to_owned())
        } else {
            None
        }
    }

    /// Otherwise, find the most common path from all the test file paths.
    ///
    /// NOTE: this will return `None` in several cases, such as if there is only one test path,
    /// or On windows in the case where there are tests located on several different device drives.
    fn most_common_test_path_ancestor(test_absolute_path: &Path, config: &Config)
        -> Option<PathBuf> {
        // different disk drives at the same time.
        {
            let initial_current_path_containing_everything_so_far = test_absolute_path.parent().unwrap();
            let mut current_path_containing_everything_so_far = initial_current_path_containing_everything_so_far;

            for test_path in config.test_paths.iter() {
                if !test_path.starts_with(current_path_containing_everything_so_far) {
                    let common_ancestor = test_path.ancestors().find(|p| current_path_containing_everything_so_far.starts_with(p));

                    if let Some(common_ancestor) = common_ancestor {
                        // The common ancestor path may be empty if the files are on different
                        // devices.
                        if common_ancestor.file_name().is_some() {
                            println!("common ancestor: {:?}", common_ancestor.file_name());
                            current_path_containing_everything_so_far = common_ancestor;
                        }
                    } else {
                        // N.B. we only ever expect no common ancestor on Windows
                        // where paths may be on different devices. This should be uncommon.
                        // We cannot use this logic to compute the relative path in this scenario.
                    }
                }
            }

            if current_path_containing_everything_so_far != initial_current_path_containing_everything_so_far {
                Some(current_path_containing_everything_so_far.to_owned())
            } else {
                None // no common prefix path could be calculated from the test paths
            }

        }
    }

    #[cfg(test)]
    mod test {
        use crate::Config;
        use std::path::Path;

        #[test]
        fn test_compute() {
            let config = Config {
                test_paths: [
                    "/home/foo/projects/cool-project/tests/",
                    "/home/foo/projects/cool-project/tests/run-pass/",
                    "/home/foo/projects/cool-project/tests/run-fail/",
                ].iter().map(|p| Path::new(p).to_owned()).collect(),
                ..Config::default()
            };

            assert_eq!(super::compute(
                    &Path::new("/home/foo/projects/cool-project/tests/run-pass/test1.txt"), &config),
                Some(Path::new("run-pass/test1.txt").to_owned()));
        }

        #[test]
        fn test_least_specific_parent_test_search_directory_path_when_all_test_paths_are_directories() {
            let config = Config {
                test_paths: [
                    "/home/foo/projects/cool-project/tests/",
                    "/home/foo/projects/cool-project/tests/run-pass/",
                    "/home/foo/projects/cool-project/tests/run-fail/",
                ].iter().map(|p| Path::new(p).to_owned()).collect(),
                ..Config::default()
            };

            assert_eq!(super::least_specific_parent_test_search_directory_path(
                    &Path::new("/home/foo/projects/cool-project/tests/run-pass/test1.txt"), &config),
                Some(Path::new("/home/foo/projects/cool-project/tests/").to_owned()));
        }

        #[test]
        fn test_least_specific_parent_test_search_directory_path_when_one_test_path_directory() {
            let config = Config {
                test_paths: [
                    "/home/foo/projects/cool-project/tests/",
                ].iter().map(|p| Path::new(p).to_owned()).collect(),
                ..Config::default()
            };

            assert_eq!(super::least_specific_parent_test_search_directory_path(
                    &Path::new("/home/foo/projects/cool-project/tests/run-pass/test1.txt"), &config),
                Some(Path::new("/home/foo/projects/cool-project/tests/").to_owned()));
        }

        #[test]
        fn test_most_common_test_path_ancestor_when_all_paths_are_absolute() {
            let config = Config {
                test_paths: [
                    "/home/foo/projects/cool-project/tests/run-pass/test1.txt",
                    "/home/foo/projects/cool-project/tests/run-pass/test2.txt",
                    "/home/foo/projects/cool-project/tests/run-fail/test3.txt",
                ].iter().map(|p| Path::new(p).to_owned()).collect(),
                ..Config::default()
            };

            assert_eq!(super::most_common_test_path_ancestor(
                    &Path::new("/home/foo/projects/cool-project/tests/run-pass/test1.txt"), &config),
                Some(Path::new("/home/foo/projects/cool-project/tests").to_owned()));
        }


        #[test]
        fn test_most_common_test_path_ancestor_when_all_paths_absolute_on_different_drives() {
            let config = Config {
                test_paths: [
                    "C:/tests/run-pass/test1.txt",
                    "C:/tests/run-pass/test2.txt",
                    "Z:/tests/run-fail/test3.txt",
                    "Z:/tests/run-fail/test4.txt",
                ].iter().map(|p| Path::new(p).to_owned()).collect(),
                ..Config::default()
            };

            assert_eq!(super::most_common_test_path_ancestor(
                    &Path::new("C:/tests/run-pass/test2.txt"), &config),
                None);
        }
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


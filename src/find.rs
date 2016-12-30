use std;
use walkdir::WalkDir;

/// Recursively finds tests for the given paths.
pub fn in_paths<'a,P>(paths: P) -> Result<Vec<String>,String>
    where P: IntoIterator<Item=&'a str> {
    let mut tests = Vec::new();

    for path in paths.into_iter() {
        let path_tests = try!(in_path(path));
        tests.extend(path_tests.into_iter());
    }

    Ok(tests)
}

pub fn in_path(path: &str)
    -> Result<Vec<String>,String> {
    let metadata = match std::fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) => return Err(format!("failed to open '{}': {}",
                                     path, e)),
    };

    if metadata.is_dir() {
        find_tests_in_dir(path)
    } else {
        Ok(vec![path.to_owned()])
    }
}

fn find_tests_in_dir(path: &str) -> Result<Vec<String>,String> {
    let tests = try!(find_files_in_dir(path)).into_iter()
                     .filter(|f| f.ends_with(".ir"))
                     .collect();
    Ok(tests)
}

fn find_files_in_dir(path: &str) -> Result<Vec<String>,String> {
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


use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod resolve;

pub type Variables = HashMap<String, String>;

pub trait VariablesExt {
    fn as_map(&self) -> &HashMap<String, String>;

    /// Gets a list of tempfile paths in the variable list.
    fn tempfile_paths(&self) -> Vec<PathBuf> {
        self.as_map().iter()
            .filter(|(k,_)| k.contains("tempfile"))
            .map(|(_,v)| Path::new(v).to_owned())
            .collect()
    }
}

impl VariablesExt for Variables {
    fn as_map(&self) -> &Self { self }
}


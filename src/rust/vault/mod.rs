use std::{collections::HashMap, path::PathBuf};

use pest::error;

use crate::parser::ParsedMDFile;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub(crate) struct File {
    pub path: PathBuf,
    pub contents: ParsedMDFile,
    pub original: String,
}

impl File {
    pub(crate) fn new(path: PathBuf, contents: String) -> Result<Self> {
        let parsed_mdfile = ParsedMDFile::new(path.clone(), contents.clone())?;
        Ok(File {
            path,
            contents: parsed_mdfile,
            original: contents,
        })
    }

    pub(crate) fn get_aliases(&self) -> Vec<&str> {
        let title: &str = self.contents.get_title();
        let file_aliases: Result<Vec<&str>> = self.contents.get_aliases();
        let mut aliases: Vec<&str> = vec![title];
        match file_aliases {
            Ok(file_aliases) => {
                for alias in file_aliases {
                    aliases.push(alias);
                }
            }
            Err(_) => {}
        }
        aliases
    }

    pub(crate) fn get_path(&self) -> &PathBuf {
        &self.path
    }
}

impl Default for File {
    fn default() -> Self {
        let contents: String = "".to_string();
        let path: PathBuf = PathBuf::new();
        File {
            path: path.clone(),
            contents: ParsedMDFile::new(path, contents.clone()).unwrap(),
            original: contents,
        }
    }
}

struct Vault {
    vault_files: HashMap<PathBuf, File>,
    errored_files: HashMap<PathBuf, Error>,
}

impl Vault {
    fn new(files: Vec<(PathBuf, String)>) -> Self {
        let mut vault_files: HashMap<PathBuf, File> = HashMap::new();
        let mut errored_files: HashMap<PathBuf, Error> = HashMap::new();
        for (path, content) in files {
            let parsed = File::new(path.clone(), content);
            match parsed {
                Ok(parsed) => {
                    vault_files.insert(path, parsed);
                }
                Err(e) => {
                    errored_files.insert(path, e);
                }
            };
        }
        Vault {
            vault_files,
            errored_files,
        }
    }
}

struct LinkableStrings<'a> {
    strings: Vec<&'a mut String>,
}

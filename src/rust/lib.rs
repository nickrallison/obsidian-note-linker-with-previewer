#![allow(unused)] // for beginning only

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    vec,
};

use js_sys::JsString;
use parser::Node;
use regex::{Regex, RegexBuilder};
use wasm_bindgen::prelude::*;

use crate::prelude::*;

mod error;
mod link_finder;
mod obsidian;
mod parser;
mod prelude;
mod settings;
mod utils;
mod vault;

#[wasm_bindgen]
pub struct ExampleCommand {
    id: JsString,
    name: JsString,
    vault: obsidian::Vault,
}

#[wasm_bindgen]
impl ExampleCommand {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> JsString {
        self.id.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: &str) {
        self.id = JsString::from(id)
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> JsString {
        self.name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: &str) {
        self.name = JsString::from(name)
    }

    pub fn callback(&self) {
        let num_files = &self.vault.getFiles().len();
        let message = format!("Number of files: {}", num_files);
        obsidian::Notice::new(&message);
    }
}

// #[wasm_bindgen]
// pub fn parse_file_to_str(content: JsString) -> JsString {
//     let content: String = content.as_string().unwrap();
//     let parsed: Result<parser::MDFile> = parser::parse_md_file_wrapper(content);

//     match parsed {
//         Ok(_) => JsString::from(format!("{:?}", parsed)),
//         Err(e) => JsString::from(format!("Error: {}", e)),
//     }
// }

#[wasm_bindgen]
pub fn onload(plugin: &obsidian::Plugin) {
    // let cmd = ExampleCommand {
    //     id: JsString::from("example"),
    //     name: JsString::from("Example"),
    //     vault: plugin.get_app().get_vault(),
    // };
    // plugin.addCommand(JsValue::from(cmd));
}

// Public Types and Public Functions

#[wasm_bindgen]
pub struct JsVault {
    files: VaultWrapper,
}

#[wasm_bindgen]
impl JsVault {
    #[wasm_bindgen(constructor)]
    pub fn new(file_paths: Vec<JsString>, file_contents: Vec<JsString>) -> JsVault {
        let file_paths: Vec<PathBuf> = file_paths
            .iter()
            .map(|file_path| PathBuf::from(f!("{}", file_path)))
            .collect();
        let file_contents: Vec<String> = file_contents
            .iter()
            .map(|file_content| f!("{}", file_content))
            .collect();
        let files = VaultWrapper::new(file_paths, file_contents);
        JsVault { files }
    }

    #[wasm_bindgen]
    pub fn default() -> Self {
        JsVault {
            files: VaultWrapper::default(),
        }
    }

    #[wasm_bindgen]
    pub fn add_file(&mut self, file_path: JsString, file_content: JsString) {
        let file_path = PathBuf::from(f!("{}", file_path));
        let file_content = f!("{}", file_content);
        self.files.add_file(file_path, file_content);
    }

    #[wasm_bindgen]
    pub fn get_file(&self, file_path: JsString) -> JsFile {
        let file_path = PathBuf::from(f!("{}", file_path));
        let file_opt: Option<&crate::vault::File> = self.files.get_file(file_path);
        let valid: bool = file_opt.is_some();
        let mut file: crate::vault::File = crate::vault::File::default();
        if valid {
            file = file_opt.unwrap().clone();
        }

        JsFile { valid, file }
    }

    #[wasm_bindgen]
    pub fn get_valid_file_paths(&self) -> Vec<JsString> {
        let file_paths: Vec<&PathBuf> = self.files.get_valid_file_paths();
        file_paths
            .iter()
            .map(|path| JsString::from(format!("{}", path.display())))
            .collect()
    }
}

#[wasm_bindgen]
pub struct JsFile {
    valid: bool,
    file: crate::vault::File,
}

#[wasm_bindgen]
pub struct JsLinkFinder {
    link_finder: LinkFinderWrapper,
}

#[wasm_bindgen]
impl JsLinkFinder {
    #[wasm_bindgen(constructor)]
    pub fn new(
        file_paths: Vec<JsString>,
        files: Vec<JsFile>,
        settings: JsSettings,
    ) -> JsLinkFinder {
        let file_paths: Vec<String> = file_paths.iter().map(|file| f!("{}", file)).collect();
        let files: Vec<&crate::vault::File> = files.iter().map(|file| &file.file).collect();
        let settings = settings.settings;

        let link_finder = LinkFinderWrapper::new(file_paths, files, settings);

        JsLinkFinder { link_finder }
    }
    #[wasm_bindgen]
    pub fn find_links(&self, file: JsFile) -> Vec<JsLink> {
        let file = file.file;
        let links: Vec<crate::link_finder::Link> = self.link_finder.find_links(file);
        links
            .iter()
            .map(|link| JsLink { link: link.clone() })
            .collect()
    }
}

#[wasm_bindgen]
pub struct JsSettings {
    settings: crate::settings::Settings,
}

#[wasm_bindgen]
impl JsSettings {
    #[wasm_bindgen(constructor)]
    pub fn new(case_insensitive: bool, link_to_self: bool, color: String) -> JsSettings {
        let settings = crate::settings::Settings::new(case_insensitive, link_to_self, color);
        JsSettings { settings }
    }

    #[wasm_bindgen]
    pub fn default() -> Self {
        let settings = crate::settings::Settings::default();
        JsSettings { settings }
    }

    #[wasm_bindgen]
    pub fn get_case_insensitive(&self) -> bool {
        self.settings.case_insensitive
    }

    #[wasm_bindgen]
    pub fn get_link_to_self(&self) -> bool {
        self.settings.link_to_self
    }

    #[wasm_bindgen]
    pub fn get_color(&self) -> JsString {
        JsString::from(self.settings.color.clone())
    }

    // #[wasm_bindgen]
    // pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
    //     self.settings.case_insensitive = case_insensitive;
    // }

    // #[wasm_bindgen]
    // pub fn set_link_to_self(&mut self, link_to_self: bool) {
    //     self.settings.link_to_self = link_to_self;
    // }

    // #[wasm_bindgen]
    // pub fn set_color(&mut self, color: JsString) {
    //     self.settings.color = f!("{}", color);
    // }
}

#[wasm_bindgen]
pub struct JsLink {
    link: link_finder::Link,
}

#[wasm_bindgen]
impl JsLink {
    #[wasm_bindgen]
    pub fn get_source(&self) -> JsString {
        JsString::from(format!("{}", self.link.source.display()))
    }
    #[wasm_bindgen]
    pub fn get_target(&self) -> JsString {
        JsString::from(format!("{}", self.link.target.display()))
    }
    #[wasm_bindgen]
    pub fn get_start(&self) -> JsValue {
        self.link.byte_start.into()
    }
    #[wasm_bindgen]
    pub fn get_end(&self) -> JsValue {
        self.link.byte_end.into()
    }
}

// Interface Types
pub struct VaultWrapper {
    valid_files: HashMap<PathBuf, crate::vault::File>,
    invalid_files: Vec<(PathBuf, Error)>,
}

impl VaultWrapper {
    fn new(file_paths: Vec<PathBuf>, file_contents: Vec<String>) -> Self {
        let mut vault_wrapper: VaultWrapper = Default::default();

        for (file_path, file_content) in file_paths.iter().zip(file_contents.iter()) {
            vault_wrapper.add_file(file_path.clone(), file_content.clone());
        }

        vault_wrapper
    }

    fn add_file(&mut self, file_path: PathBuf, file_content: String) {
        match crate::vault::File::new(file_path.clone(), file_content.clone()) {
            Ok(file) => {
                self.valid_files.insert(file_path.clone(), file);
            }
            Err(e) => {
                self.invalid_files.push((file_path.clone(), e));
            }
        }
    }

    fn get_file(&self, file_path: PathBuf) -> Option<&crate::vault::File> {
        self.valid_files.get(&file_path)
    }

    fn get_valid_file_paths(&self) -> Vec<&PathBuf> {
        self.valid_files.keys().collect()
    }
}

impl Default for VaultWrapper {
    fn default() -> Self {
        VaultWrapper {
            valid_files: HashMap::new(),
            invalid_files: Vec::new(),
        }
    }
}

// wrapper around the LinkFinder class
#[derive(Debug)]
pub struct LinkFinderWrapper {
    link_finder: link_finder::LinkFinder,
}

impl LinkFinderWrapper {
    pub fn new(
        file_paths: Vec<String>,
        files: Vec<&crate::vault::File>,
        settings: crate::settings::Settings,
    ) -> LinkFinderWrapper {
        let file_paths: Vec<PathBuf> = file_paths
            .iter()
            .map(|file_path| PathBuf::from(file_path))
            .collect();

        let file_refs: Vec<&crate::vault::File> = files
            .iter()
            .map(|file: &&crate::vault::File| *file)
            .collect();

        let link_finder = link_finder::LinkFinder::new(file_refs, settings);
        LinkFinderWrapper { link_finder }
    }

    pub fn find_links(&self, file: crate::vault::File) -> Vec<link_finder::Link> {
        self.link_finder.get_links(&file)
    }
}

const FILE_1_PATH: &str = "alan turing.md";
const FILE_1_CONT: &str = r#"---
bad_links: 
tags: [computerscience]
date created: Monday, July 10th 2023, 12:23:57 am
title: Alan Turing
aliases: ["turing"]
---

# Alan Turing

Alan Mathison Turing, the father of the Turing Machine was born on June 23, 1912, in London. He is widely considered as the father of theoretical computer science and artificial intelligence. Turing studied mathematics at King's College, University of Cambridge, where he developed the concept of a "universal machine" that could compute anything that is computable. This idea formed the basis of all modern computers.  
During World War II, Turing worked at Bletchley Park, Britain's codebreaking centre, and was instrumental in breaking the German Enigma code. His work is said to have significantly shortened the war and saved countless lives.  
Post war, Turing worked on developing an early computer at the National Physical Laboratory and later on artificial intelligence at the University of Manchester. He proposed an experiment now known as the "Turing Test" to determine if a machine can exhibit intelligent behavior equivalent to or indistinguishable from human behavior.  
Despite his accomplishments, Turing faced persecution for his homosexuality - which was illegal in Britain at that time. He was convicted for "gross indecency" in 1952 and underwent chemical castration as an alternative to prison. Tragically, this led to his untimely death by suicide on June 7, 1954.  
In 2013, Queen Elizabeth II granted Turing a posthumous royal pardon. His legacy continues through the "Turing Award", often referred to as the 'Nobel Prize' of computing world which is given annually by ACM (Association for Computing Machinery). In addition, his life and work have been depicted in various forms including the 2014 film "The Imitation Game"."#;

const FILE_2_PATH: &str = "turing machine.md";
const FILE_2_CONT: &str = r#"---
bad_links: 
aliases: []
tags: [computerscience, theoreticalcompsci]
title: Turing Machine
date created: Monday, July 24th 2023, 7:44:20 pm
---
# Turing Machine

A Turing Machine is a theoretical computational device, conceived by British mathematician Alan Turing in 1936. Its an abstract model of computation that manipulates symbols on a strip of tape according to a table of rules. Despite its simplicity, a Turing machine can simulate the logic of any computer algorithm and is used in theoretical computer science to understand what can be computed. Its also a key concept in the theory of computation and computability."#;

#[cfg(test)]
pub mod wasm_test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn wasm_alan_turing_test() {
        let settings = crate::settings::Settings::new(true, false, "red".to_string());
        let file_1_path: PathBuf = PathBuf::from(FILE_1_PATH);
        let file_2_path: PathBuf = PathBuf::from(FILE_2_PATH);
        let file1 = crate::vault::File::new(file_1_path, FILE_1_CONT.to_string()).unwrap();
        let file2 = crate::vault::File::new(file_2_path, FILE_2_CONT.to_string()).unwrap();
        let files = vec![&file1, &file2];
        let link_finder = LinkFinderWrapper::new(
            vec![FILE_1_PATH.to_string(), FILE_2_PATH.to_string()],
            files,
            settings,
        );
        let links = link_finder.find_links(file1);
        let links_expected: Vec<crate::link_finder::Link> = vec![crate::link_finder::Link {
            source: PathBuf::from(FILE_1_PATH),
            target: PathBuf::from(FILE_2_PATH),
            byte_start: 189,
            byte_end: 203,
        }];
        assert_eq!(links, links_expected);
    }

    #[test]
    fn wasm_turing_machine_test() {
        let settings = crate::settings::Settings::new(true, false, "red".to_string());
        let file_1_path: PathBuf = PathBuf::from(FILE_1_PATH);
        let file_2_path: PathBuf = PathBuf::from(FILE_2_PATH);
        let file1 = crate::vault::File::new(file_1_path, FILE_1_CONT.to_string()).unwrap();
        let file2 = crate::vault::File::new(file_2_path, FILE_2_CONT.to_string()).unwrap();
        let files = vec![&file1, &file2];
        let link_finder = LinkFinderWrapper::new(
            vec![FILE_1_PATH.to_string(), FILE_2_PATH.to_string()],
            files,
            settings,
        );
        let links = link_finder.find_links(file2);
        /*
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 149, byte_end: 155 }
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 167, byte_end: 173 }
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 256, byte_end: 267 }
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 415, byte_end: 421 }
        */
        let links_expected: Vec<crate::link_finder::Link> = vec![
            crate::link_finder::Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 149,
                byte_end: 155,
            },
            crate::link_finder::Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 167,
                byte_end: 173,
            },
            crate::link_finder::Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 256,
                byte_end: 267,
            },
            crate::link_finder::Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 415,
                byte_end: 421,
            },
        ];
        assert_eq!(links, links_expected);
    }
}

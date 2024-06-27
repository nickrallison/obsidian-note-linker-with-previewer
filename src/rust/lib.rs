#![allow(unused)] // for beginning only

mod obsidian;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    vec,
};

use js_sys::JsString;
use wasm_bindgen::prelude::*;

use crate::prelude::*;

mod error;
mod parser;
mod prelude;
mod utils;

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

#[wasm_bindgen]
pub struct JsLinker {
    files: Vec<Result<parser::MDFile>>,
}

#[wasm_bindgen]
impl JsLinker {
    #[wasm_bindgen(constructor)]
    pub fn new(file_paths: Vec<JsString>, file_contents: Vec<JsString>) -> Self {
        // file_map is a map of file paths to file contents

        let mut md_files: Vec<Result<parser::MDFile>> = vec![];
        for (path, content) in file_paths.iter().zip(file_contents.iter()) {
            let content: String = content.as_string().unwrap();
            let path: String = path.as_string().unwrap();
            md_files.push(parser::parse_md_file_wrapper(content, path));
        }

        JsLinker { files: md_files }
    }
    #[wasm_bindgen]
    pub fn get_bad_parse_files(&self) -> Vec<JsString> {
        let mut bad_files: Vec<JsString> = vec![];
        for file in &self.files {
            match file {
                Ok(_) => (),
                Err(e) => match e {
                    Error::ParseError(path, error) => {
                        bad_files.push(JsString::from(format!("{}", path.display())))
                    }
                    _ => (),
                },
            }
        }
        bad_files
    }
    #[wasm_bindgen]
    pub fn get_links(&self) -> Vec<JsLink> {
        let mut links: Vec<JsLink> = vec![];

        let mut alias_map: HashMap<&Path, Vec<&str>> = HashMap::new();

        for file in &self.files {
            let mut aliases: Vec<&str> = vec![];
            match file {
                Ok(md_file) => {
                    let title: &str = md_file.get_title();
                    aliases.push(title);
                    let file_aliases: Result<Vec<&str>> = md_file.get_aliases();
                    let mut aliases: Vec<&str> = vec![title];
                    match file_aliases {
                        Ok(file_aliases) => {
                            for alias in file_aliases {
                                aliases.push(alias);
                            }
                        }
                        Err(_) => {}
                    }
                    alias_map.insert(&md_file.path, aliases);
                }
                Err(_) => (),
            }
        }
        let debug_link = JsLink {
            debug_field: format!("{:?}", alias_map),
        };
        links.push(debug_link);
        links
    }
}

#[wasm_bindgen]
pub struct JsLink {
    debug_field: String,
    // source: String,
    // target: String,
    // link_text: String,
}

#[wasm_bindgen]
impl JsLink {
    #[wasm_bindgen]
    pub fn debug(&self) -> String {
        self.debug_field.clone()
    }
}

// use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet_rust(name: &str) {
//     alert(&format!("Hello, {}!", name));
// }

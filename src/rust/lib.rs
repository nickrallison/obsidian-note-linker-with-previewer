#![allow(unused)] // for beginning only

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    vec,
};

use js_sys::JsString;
use regex::{Regex, RegexBuilder};
use wasm_bindgen::prelude::*;

use crate::prelude::*;

mod error;
mod obsidian;
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
    pub fn get_links(&self, options: obsidian::RustPluginSettings) -> Vec<JsLink> {
        let mut links: Vec<JsLink> = vec![];

        // constructing a list of aliases for constructing the replace regex
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

        // constructing the replace regex
        // ((?:Elliptic Curve Cryptography)|(?:Elliptic Curve))|((?:identity element)|(?:identity))
        // all of the aliases from a file are grouped together

        // the regex group of each file is contained here
        // for example the group index of Elliptic Curve Cryptography would be 0, ...
        // ...and the group index of identity element would be 1

        let mut file_groups: HashMap<&Path, u32> = HashMap::new();
        for (path, aliases) in alias_map.iter() {
            let mut group_index: u32 = 0;
            file_groups.insert(path, group_index);
        }

        let mut file_regex_strs: Vec<String> = vec![];
        for (path, aliases) in alias_map.iter() {
            let mut file_regex_str: String = String::from("(");
            for alias in aliases {
                file_regex_str.push_str(&format!("(?:{})|", alias));
            }
            file_regex_str.pop();
            file_regex_str.push(')');
            file_regex_strs.push(file_regex_str);
        }

        let regex: String = file_regex_strs.join("|");
        // if case_sensitive {

        let regex: Regex = RegexBuilder::new(&regex)
            .case_insensitive(options.get_case_insensitive())
            .build()
            .expect("Invalid Regex");

        for file in &self.files {
            match file {
                Ok(md_file) => {
                    let string_nodes: Vec<crate::parser::StringPosition> =
                        md_file.get_string_nodes();
                    // let links: Vec<JsLink> = JsLinker::get_links_from_file(
                    //     content,
                    //     &regex,
                    //     &alias_map,
                    //     &file_groups,
                    //     &md_file.path,
                    // );
                    // for link in links {
                    //     links.push(link);
                    // }
                }
                Err(_) => (),
            }
        }

        // let debug_link = JsLink {
        //     debug_field: format!("regex_p: {:?}", regex_p),
        // };
        // links.push(debug_link);
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

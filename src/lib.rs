#![allow(unused)] // for beginning only

mod obsidian;
use std::{io::Result, result};

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
        obsidian::Notice::new("hello from rust");
    }
}

#[wasm_bindgen]
pub fn parse_files_to_str(vault: &obsidian::Vault) -> JsString {
    let files = vault.getFiles();
    let len_files = files.len();
    JsString::from(format!("Number of files: {}", len_files))
}

#[wasm_bindgen]
pub fn onload(plugin: &obsidian::Plugin) {
    let cmd = ExampleCommand {
        id: JsString::from("example"),
        name: JsString::from("Example"),
    };
    // plugin.addCommand(JsValue::from(cmd));
}

#[wasm_bindgen]
struct ExampleStruct {
    id: String,
}

#[wasm_bindgen]
impl ExampleStruct {
    #[wasm_bindgen(constructor)]
    pub fn new(id: JsString) -> ExampleStruct {
        ExampleStruct {
            id: id.as_string().unwrap(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> JsString {
        self.id.clone().into()
    }
    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: JsString) {
        self.id = id.as_string().unwrap();
    }
    #[wasm_bindgen]
    pub fn do_thing(&self) -> JsString {
        let mut temp = self.id.clone();
        temp.push_str(" is the id");
        temp.into()
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

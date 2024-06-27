#![allow(unused)] // for beginning only

mod obsidian;

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

#[wasm_bindgen]
pub fn parse_file_to_str(content: JsString) -> JsString {
    let content: String = content.as_string().unwrap();
    let parsed: Result<parser::MDFile> = parser::parse_md_file_wrapper(content);

    match parsed {
        Ok(_) => JsString::from(format!("{:?}", parsed)),
        Err(e) => JsString::from(format!("Error: {}", e)),
    }
}

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

#![allow(unused)] // for beginning only

mod obsidian;
use js_sys::JsString;
use wasm_bindgen::prelude::*;

use crate::prelude::*;

mod error;
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
pub struct ParseCommand {
    id: JsString,
    name: JsString,
}

#[wasm_bindgen]
impl ParseCommand {
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
        obsidian::Notice::new("hello from parse 1");
    }
}

#[wasm_bindgen]
pub fn onload(plugin: &obsidian::Plugin) {
    let cmd = ExampleCommand {
        id: JsString::from("example"),
        name: JsString::from("Example"),
    };
    plugin.addCommand(JsValue::from(cmd));

    let cmd = ParseCommand {
        id: JsString::from("parse"),
        name: JsString::from("Parse"),
    };
    plugin.addCommand(JsValue::from(cmd));
}

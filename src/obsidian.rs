use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "obsidian")]
extern "C" {
    pub type Plugin;
    pub type App;
    pub type Vault;
    pub type TFile;
    pub type Notice;

    #[wasm_bindgen(structural, method)]
    pub fn addCommand(this: &Plugin, command: JsValue);

    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Notice;

    #[wasm_bindgen(structural, method)]
    pub fn getFiles(vault: &Vault) -> Vec<TFile>;
}

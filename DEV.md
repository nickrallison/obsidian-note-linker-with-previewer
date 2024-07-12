

# Developement

Build instructions:
```bash
cd [pluginfolder]

# only need to run this once
yarn install

yarn run dev
```
if you get this error after running `yarn run dev`, just rerun the command and it should resolve on its own

Error:
`ENOENT: no such file or directory, open '... obsidian-note-linker-with-previewer/target/wasm32-unknown-unknown/release/obsidian_note_linker_with_previewer.d'`



## Dev Requirements

- wsl / linux work for building, windows & macos are untested
- cargo
- wasm-pack
- yarn
# Obsidian Linker Plugin
## Usage
### Summary

This is a plugin to find links between your notes. That's it, that's all it does. 

#### Features

- Doesn't link inside code blocks, latex blocks, or other special markdown tools, it does this as it has a built in markdown parser
	- If you encounter any links placed in bad locations, please open a ticket and I'll fix it as soon as I am able.

![2024-07-12 14-31-18](https://github.com/user-attachments/assets/24c5b38b-5e8b-4d72-95e4-ee91f0cae5b6)

- "Fast" Linking. Searching for links is an intensive operation, this is the most intensive part of this plugin. It uses caching to reuse up to date links, so rerunning the command to link your vault will be much faster on the second run.

### Directions

You have 3 commands you can use
1. Scan Vault
	- I recommend running this first & letting it complete. This searches for links and records them without writing any files. Subsequent links runs will be much faster. The saved results should also persist when you close the app
2. Link Vault
	- This command links every markdown file in your vault
 	- If you specify an include filter, it only searches files matching one or more of those filters
3. Link Current Note
	- This only links your current note
 	- This bypasses the filter setting.


## Developement

```bash
cd [pluginfolder]

# only need to run this once
yarn install

yarn run dev
```
if you get this error after running `yarn run dev`, just rerun the command and it should resolve on its own
ENOENT: no such file or directory, open '... obsidian-note-linker-with-previewer/target/wasm32-unknown-unknown/release/obsidian_note_linker_with_previewer.d'

### Developer Requirements

- wsl / linux work for building, windows & macos are untested
- cargo
- wasm-pack
- yarn

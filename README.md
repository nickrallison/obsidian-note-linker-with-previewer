# Obsidian Linker Plugin

## How to Use


## Developement

### Building

```bash
cd [pluginfolder]

# only need to run this once
yarn install

yarn run dev
```

### Dev Requirements

- cargo
- wasm-pack
- yarn

### Todo

- Performance
    - Serde and Selectively Searching based on last modified metadata
    - Non-Blocking search, so it keeps going in background
    - Completely Background search
- Organization
	- Add CI Build
    - Add Nix Flake to build more easily 

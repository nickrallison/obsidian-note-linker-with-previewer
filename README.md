# Obsidian Linker Plugin
## Usage
### Summary

This is a plugin to find links between your notes. That's it, that's all it does.

![2024-07-12 14-31-18](https://github.com/user-attachments/assets/5c3170db-0e39-4ac1-83bb-f0a183b30478)

#### Features

- Doesn't link inside code blocks, latex blocks, or other special markdown tools, it does this as it has a built in markdown parser
	- If you encounter any links placed in bad locations, please open a ticket and I'll fix it as soon as I am able.

- Link Caching, Searching for links is an intensive operation, this is the most intensive part of this plugin. It uses caching to reuse up to date links, so rerunning the command to link your vault will be much faster on the second run.

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

Please see the [developer documentation](DEV.md)


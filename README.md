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

You have several commands you can use
1. Scan Vault
	- I recommend running this first & letting it complete. This searches for links and records them without writing any files. Subsequent links runs will be much faster. The saved results should also persist when you close the app
2. Link Vault
	- This command links every markdown file in your vault
	- If you specify an include filter, it only searches files matching one or more of those filters
3. Link Current Note
	- This only links your current note
	- This bypasses the filter setting
4. Get Invalid Notes
	- This shows all of the files which had an error and are not actively being scanned
	- **This one is important. Read the section below**
5. Reset Cache
	- This is helpful if you are getting weird linking behavior, it hard resets your cache so when it searches again, it searches from scratch.

### Parsing & Invalid Notes

Since a parser is used to link your notes, it won't suggest links inside of code blocks, or other obviously wrong sections. That comes with a downside, if your notes are not properly formatted, they will not parse and as a result, they will not be linked, and nothing will link to them.

#### How Can You Avoid This

##### Obvious bad formatting will cause an error:
eg: "[[Vector.md|Binormal]] Vector.md|Binormal]]"
##### Escape Characters
The follow characters have to be escaped when used in normal text sections:
- "*"
- ">"
- "["
- "]"
- "$"

##### Code Blocks
Code Blocks and Latex blocks must start with a new line and end with a new line. Whitespaces at the beginning of a line cause parsing to fail

Bad:
~~~
         ```c
#include <stdio.h>
#include <stdlib.h>
         ```
~~~
Good:
~~~
```c
#include <stdio.h>
#include <stdlib.h>
```
~~~


##### Termination

Special characters must be terminated.

Good:
- `This is **Important** text`

Bad:
- `This is **Important text`

Good:
- `This is a [[link]] to a page`

Bad:
- `This is a [[link to a page`
- `This is a link]] to a page`

#### My Notes are Still Getting an Error

I am so sorry for the frustration. You have 2 options.

1. You can open an interactive viewer of how your file is being parsed, and debug it yourself.
	1. You can find it [here](https://pest.rs/#editor). It needs a [grammar](https://github.com/nickrallison/obsidian-note-linker-with-previewer/blob/main/src/rust/parser/md.pest), and an input file.
	2. Give it the contents of the grammer and the contents of your failing file
	3. Choose to parse it into an md_file under the input mode
	4. If you do this, **Make sure to add a newline at the end of your file manually**, otherwise it may not parse
2. If you think you have found a bug, please open a pull request with the following information. I will do my best to fix it as fast as I can
	1. The version of your plugin
	2. The contents of the file you are parsing


## Developement

Please see the [developer documentation](DEV.md)


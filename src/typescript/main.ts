import {
  App,
  MarkdownRenderer,
  MarkdownView,
  Modal,
  Notice,
  Plugin,
  PluginSettingTab,
  Setting,
  TFile,
} from "obsidian";
import * as plugin from "../../pkg/obsidian_note_linker_with_previewer.js";

// DO NOT REMOVE, PLUGIN DOES NOT LOAD WITHOUT IT
import rustPlugin from "../../pkg/obsidian_note_linker_with_previewer_bg.wasm";

class RustPluginSettings {
  caseInsensitive: boolean;
  color: string;
  includePaths: string;
  excludePaths: string;

  constructor(caseInsensitive: boolean, color: string) {
    this.caseInsensitive = caseInsensitive;
    this.color = color;
    this.includePaths = "";
  }
  set_case_insensitive(caseSensitive: boolean) {
    this.caseInsensitive = caseSensitive;
  }
  get_case_insensitive() {
    return this.caseInsensitive;
  }
}

const DEFAULT_SETTINGS: RustPluginSettings = new RustPluginSettings(
  true,
  "red",
);

interface FileChange {
  file_path: string;
  new_content: string;
  colored_content: string;
}

export default class RustPlugin extends Plugin {
  settings: RustPluginSettings;
  cache_path: string =
    this.manifest.dir +
    "/cache.json";
  cache_obj: { [key: string]: any } = {};

  async onload() {
    await this.loadSettings();

    // DO NOT REMOVE, PLUGIN DOES NOT LOAD WITHOUT IT
    const buffer = Buffer.from(rustPlugin, "base64");
    // DO NOT REMOVE, PLUGIN DOES NOT LOAD WITHOUT IT
    await plugin.default(Promise.resolve(buffer));

    this.cache_obj = {};
    await this.read_cache();

    this.addCommand({
      id: "link_current_file",
      name: "Link current note",
      callback: () => {
        let active_view = this.app.workspace.getActiveViewOfType(MarkdownView);
        if (active_view) {
          this.run_linker_on_file(active_view.file.path);
        }
      },
    });

    this.addCommand({
      id: "link_vault",
      name: "Link vault",
      callback: () => {
        this.run_linker();
      },
    });

    this.addCommand({
      id: "scan_vault",
      name: "Scan vault",
      callback: () => {
        this.scan_vault();
      },
    });

    this.addCommand({
      id: "invalid_notes",
      name: "Get invalid notes",
      callback: () => {
        this.get_invalid_notes();
      },
    });

    this.addCommand({
      id: "reset_cache",
      name: "Reset cache",
      callback: async () => {
        this.cache_obj = {};
        await this.write_cache();
      },
    });

    // this.addCommand({
    // 	id: "debug",
    // 	name: "Debug",
    // 	callback: async () => {
    // 		let config_dir = this.app.vault.configDir + '/plugins/obsidian-note-linker-with-previewer';
    // 		let cache_files = await this.app.vault.adapter.list(config_dir);
    // 		console.log(cache_files);
    // 	}
    // });

    this.addSettingTab(new RustPluginSettingTab(this.app, this));
  }

  async run_linker() {
    await this.save_active_file();
    let tfilemap: { [key: string]: TFile } = await this.get_filtered_filemap();
    let file_paths: string[] = Object.keys(tfilemap);
    let wasm_vault: plugin.JsVault = await this.create_wasm_vault(tfilemap);
    let alias_map: { [key: string]: string[] } =
      await this.get_alias_map(tfilemap);

    let files: plugin.JsFile[] = [];
    for (let file_path of file_paths) {
      files.push(wasm_vault.get_file(file_path));
    }
    // let settings_obj = new plugin.JsSettings(this.settings.caseInsensitive, link_to_self, this.settings.color);
    let valid_file_paths: string[] = wasm_vault.get_valid_file_paths();
    let valid_files: plugin.JsFile[] = valid_file_paths.map((path) =>
      wasm_vault.get_file(path),
    );
    let link_finder: plugin.JsLinkFinder = new plugin.JsLinkFinder(
      valid_file_paths,
      valid_files,
      this.settings.caseInsensitive,
    );

    let valid_files_len = valid_files.length;
    let valid_index = 1;

    this.validate_cache(valid_file_paths, tfilemap);

    for (let file_path of valid_file_paths) {
      await this.process_file(
        file_path,
        tfilemap,
        link_finder,
        wasm_vault,
        alias_map,
        valid_index,
        valid_files_len,
        true,
      );
      valid_index++;
    }
  }

  async run_linker_on_file(active_file_path: string) {
    await this.save_active_file();

    let tfilemap: { [key: string]: TFile } = await this.get_filemap();
    let file_paths: string[] = Object.keys(tfilemap);
    let wasm_vault: plugin.JsVault = await this.create_wasm_vault(tfilemap);
    let alias_map: { [key: string]: string[] } =
      await this.get_alias_map(tfilemap);

    let files: plugin.JsFile[] = [];
    for (let file_path of file_paths) {
      files.push(wasm_vault.get_file(file_path));
    }
    let valid_file_paths: string[] = wasm_vault.get_valid_file_paths();
    let valid_files: plugin.JsFile[] = valid_file_paths.map((path) =>
      wasm_vault.get_file(path),
    );
    let link_finder: plugin.JsLinkFinder = new plugin.JsLinkFinder(
      valid_file_paths,
      valid_files,
      this.settings.caseInsensitive,
    );

    this.validate_cache(valid_file_paths, tfilemap);

    await this.process_file(
      active_file_path,
      tfilemap,
      link_finder,
      wasm_vault,
      alias_map,
      1,
      1,
      true,
    );
  }

  async scan_vault() {
    await this.save_active_file();
    let tfilemap: { [key: string]: TFile } = await this.get_filtered_filemap();
    let file_paths: string[] = Object.keys(tfilemap);
    let wasm_vault: plugin.JsVault = await this.create_wasm_vault(tfilemap);
    let alias_map: { [key: string]: string[] } =
      await this.get_alias_map(tfilemap);

    let files: plugin.JsFile[] = [];
    for (let file_path of file_paths) {
      files.push(wasm_vault.get_file(file_path));
    }
    let valid_file_paths: string[] = wasm_vault.get_valid_file_paths();
    let valid_files: plugin.JsFile[] = valid_file_paths.map((path) =>
      wasm_vault.get_file(path),
    );
    let link_finder: plugin.JsLinkFinder = new plugin.JsLinkFinder(
      valid_file_paths,
      valid_files,
      this.settings.caseInsensitive,
    );

    let valid_files_len = valid_files.length;
    let valid_index = 1;

    this.validate_cache(valid_file_paths, tfilemap);

    for (let file_path of valid_file_paths) {
      await this.process_file(
        file_path,
        tfilemap,
        link_finder,
        wasm_vault,
        alias_map,
        valid_index,
        valid_files_len,
        false,
      );
      valid_index++;
    }
  }

  async get_invalid_notes() {
    await this.save_active_file();
    let tfilemap: { [key: string]: TFile } = await this.get_filtered_filemap();
    let file_paths: string[] = Object.keys(tfilemap);
    let wasm_vault: plugin.JsVault = await this.create_wasm_vault(tfilemap);

    let invalid_files: plugin.JsFileError[] = wasm_vault.get_invalid_files();
    let paths: string[] = invalid_files.map((file) => file.get_path());
    let errors: string[] = invalid_files.map((file) => file.get_error());
    let modal = new ParseErrorModal(this, paths, errors);
    modal.open();
  }

  async loadSettings() {
    this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
  }

  async saveSettings() {
    await this.saveData(this.settings);
  }

  async get_aliases(file: TFile): Promise<string[]> {
    let frontmatter = (await this.app.metadataCache.getFileCache(file))
      ?.frontmatter;
    let aliases: string[] = [];
    if (frontmatter) {
      if (frontmatter.aliases) {
        aliases = frontmatter.aliases;
      }
    }
    let basename = file.basename;
    if (!aliases.includes(basename)) {
      aliases.push(basename);
    }
    return aliases;
  }

  async validate_cache(
    valid_file_paths: string[],
    tfilemap: { [key: string]: TFile },
  ) {
    this.cache_obj = JSON.parse(
      await this.app.vault.adapter.read(this.cache_path),
    );
    let alias_map: { [key: string]: string[] } = {};

    let index = 0;
    let valid_files_len = valid_file_paths.length;
    for (index = 0; index < valid_files_len; index++) {
      let file_path = valid_file_paths[index];
      alias_map[file_path] = await this.get_aliases(tfilemap[file_path]);
    }
    let cache_valid = true;
    for (index = 0; index < valid_files_len; index++) {
      let file_path = valid_file_paths[index];
      // if all current file aliases match the cache do nothing, otherwise clear cache
      if (
        this.cache_obj[file_path] == undefined ||
        this.cache_obj[file_path]["aliases"] == undefined
      ) {
        cache_valid = false;
        break;
      }
      let cache_aliases_set = new Set(this.cache_obj[file_path]["aliases"]);
      let current_aliases = alias_map[file_path];
      for (let alias of current_aliases) {
        // assert alias is in cache
        if (!cache_aliases_set.has(alias)) {
          cache_valid = false;
          break;
        }
      }
    }

    if (cache_valid) {
      // console.log("Cache is valid");
    } else {
      // console.log("Cache is invalid");
      this.cache_obj = {};
      await this.write_cache();
    }

    for (index = 0; index < valid_files_len; index++) {
      let file_path = valid_file_paths[index];
      if (!this.cache_obj[file_path]) {
        this.cache_obj[file_path] = {};
      }
      if (!this.cache_obj[file_path]["aliases"]) {
        this.cache_obj[file_path]["aliases"] = {};
      }
      this.cache_obj[file_path]["aliases"] = alias_map[file_path];
    }

    await this.write_cache();
  }

  async get_links(
    tfile: TFile,
    link_finder: plugin.JsLinkFinder,
    file: plugin.JsFile,
  ): Promise<plugin.JsLink[]> {
    let file_links: plugin.JsLink[];
    let cache_string: string = await this.app.vault.adapter.read(
      this.cache_path,
    );
    let cache_obj = JSON.parse(cache_string);
    let file_path = tfile.path;
    if (cache_obj[file_path]) {
      let last_modified = cache_obj[file_path]["time"];
      let file_last_modified = tfile.stat.mtime;
      // if last modified time is the same or earlier as cache
      if (file_last_modified <= last_modified) {
        let file_links_serialized: string[] = cache_obj[file_path]["links"];
        file_links = file_links_serialized.map(
          (link_serialized) => new plugin.JsLink(link_serialized),
        );
      } else {
        file_links = link_finder.find_links(file);
      }
    } else {
      file_links = link_finder.find_links(file);
    }
    return file_links;
  }

  async save_active_file() {
    let active_view = this.app.workspace.getActiveViewOfType(MarkdownView);
    if (active_view) {
      await active_view.save();
    }
  }

  async validate_file(tfile: TFile): Promise<boolean> {
    let file_path = tfile.path;
    let valid: boolean =
      this.cache_obj[file_path] &&
      this.cache_obj[file_path]["links"] &&
      this.cache_obj[file_path]["links"].length == 0 &&
      tfile.stat.mtime <= this.cache_obj[file_path]["time"];
    return valid;
  }

  async get_filemap(): Promise<{ [key: string]: TFile }> {
    let filelist: TFile[] = this.app.vault.getMarkdownFiles();
    let filemap: { [key: string]: TFile } = {};
    for (let file of filelist) {
      filemap[file.path] = file;
    }

    return filemap;
  }
  async get_filtered_filemap(): Promise<{ [key: string]: TFile }> {
    // getting the include paths
    let includePaths: string[];
    if (this.settings.includePaths == null || this.settings.includePaths == "") {
      includePaths = [];
    } else {
      includePaths = this.settings.includePaths.split("\n");
    }

    // get exclude paths
    let excludePaths: string[];
    if (this.settings.excludePaths == null || this.settings.excludePaths == "") {
      excludePaths = [];
    } else {
      excludePaths = this.settings.excludePaths.split("\n");
    }

    // if include paths is empty, include all files

    // otherwise only add files that are prefixed by one of the include paths
    let filelist: TFile[] = this.app.vault.getMarkdownFiles();

    let filtered_filelist: TFile[] = [];
    for (let file of filelist) {
      if (includePaths.length == 0) {
        if (excludePaths.length == 0) {
          filtered_filelist.push(file);
        }
        else {
          for (let exclude_path of excludePaths)
            if (!file.path.startsWith(exclude_path)) {
              filtered_filelist.push(file);
            }
        }
      }
      else {
        if (excludePaths.length == 0) {
          for (let include_path of includePaths)
            if (file.path.startsWith(include_path)) {
              filtered_filelist.push(file);
            }
        } else {
          for (let include_path of includePaths) {
            for (let exclude_path of excludePaths)
              if (
                file.path.startsWith(include_path) &&
                !file.path.startsWith(exclude_path)
              ) {
                filtered_filelist.push(file);
              }
          }
        }
      }

    }

    let filtered_filemap: { [key: string]: TFile } = {};
    for (let file of filtered_filelist) {
      filtered_filemap[file.path] = file;
    }
    return filtered_filemap;
  }
  async get_alias_map(filemap: {
    [key: string]: TFile;
  }): Promise<{ [key: string]: string[] }> {
    let alias_map: { [key: string]: string[] } = {};
    let filelist: TFile[] = Object.values(filemap);
    for (let file of filelist) {
      alias_map[file.path] = await this.get_aliases(file);
    }
    return alias_map;
  }
  async create_wasm_vault(filemap: {
    [key: string]: TFile;
  }): Promise<plugin.JsVault> {
    let wasm_vault: plugin.JsVault = plugin.JsVault.default();
    let index = 1;
    for (let file of Object.values(filemap)) {
      wasm_vault.add_file(file.path, await this.app.vault.cachedRead(file));
      index++;
    }
    return wasm_vault;
  }
  async get_parsed_file(
    path: string,
    vault: plugin.JsVault,
  ): Promise<plugin.JsFile> {
    let file = vault.get_file(path);
    return file;
  }

  async process_file(
    file_path: string,
    tfilemap: { [key: string]: TFile },
    link_finder: plugin.JsLinkFinder,
    wasm_vault: plugin.JsVault,
    alias_map: { [key: string]: string[] },
    valid_index: number,
    valid_files_len: number,
    perform_link: boolean,
  ) {
    // file file is active file, save
    let active_view = this.app.workspace.getActiveViewOfType(MarkdownView);
    if (active_view) {
      if (file_path == active_view.file.path) {
        await this.save_active_file();
      }
    }

    if (await this.validate_file(tfilemap[file_path])) {
      // console.log(
      //   `(${valid_index} / ${valid_files_len}) up to date ` + file_path,
      // );
      valid_index++;
      return;
    }
    let byte_increament: number = 0;
    let file: plugin.JsFile = wasm_vault.get_file(file_path);
    let file_links: plugin.JsLink[] = await this.get_links(
      tfilemap[file_path],
      link_finder,
      file,
    );
    // console.log(
    //   `(${valid_index} / ${valid_files_len}) Found Links for ` + file_path,
    // );
    new Notice(
      `(${valid_index} / ${valid_files_len}) Found Links for ` + file_path,
    );
    valid_index++;
    let remaining_links: string[] = [];
    let file_content: string = await this.app.vault.cachedRead(
      tfilemap[file_path],
    );
    let file_open: boolean = false;
    let view = this.app.workspace.getActiveViewOfType(MarkdownView);
    if (view) {
      if (view.file.path == file_path) {
        file_open = true;
      }
    }
    if (view && file_open) {
      file_content = view.editor.getValue();
    }

    let current_file_text = file_content;
    // bad_links:
    // - Aliasing.md
    // - Antisymmetric Relation.md
    // get array of links: [Aliasing.md, Antisymmetric Relation.md]

    let regex = /bad_links:\n(\s+- [^\n]*)+/g;
    let bad_links = current_file_text.match(regex);
    let captured_links: string[] = [];
    let capture_regex = / - ([^\n]*)/g;
    // if badlinks is found store the capture group 1 in captured_links
    if (bad_links) {
      for (let bad_link of bad_links) {
        let match: RegExpExecArray | null;
        while ((match = capture_regex.exec(bad_link))) {
          captured_links.push(match[1]);
        }
      }
    }

    // console.log("captured_links: ", captured_links);

    let accept_all: { [key: string]: boolean } = {};
    let decline_all: { [key: string]: boolean } = {};

    for (let link of file_links) {
      let slice_start = byte_increament + link.get_start();
      let slice_end = byte_increament + link.get_end();
      let source = link.get_source();
      let target = link.get_target();
      let break_loop = false;
      // console.log("target: ", target);
      for (let captured_link of captured_links) {
        // if the basename of the target is in the captured links, skip
        // console.log("captured_link: ", captured_link);
        // console.log("target.endsWith(captured_link): ", target.endsWith(captured_link));
        if (target.endsWith(captured_link)) {
          break_loop = true;
          break;
        }
      }
      if (break_loop) {
        continue;
      }

      let encoder = new TextEncoder();
      let decoder = new TextDecoder();
      let byteArray = encoder.encode(file_content);

      let content_as_bytes = encoder.encode(file_content);

      let slicedArray = byteArray.slice(slice_start, slice_end);
      let slice_str = decoder.decode(slicedArray);
      let replace_str = `[[${target}|${slice_str}]]`;

      let replaced_as_bytes = encoder.encode(replace_str);
      let increment = replaced_as_bytes.length - (slice_end - slice_start);

      let color = this.settings.color;
      let colored_content =
        decoder.decode(content_as_bytes.slice(0, slice_start)) +
        `<span style="color:${color}">\\[\\[${target}\\|${slice_str}\\]\\]</span>` +
        decoder.decode(content_as_bytes.slice(slice_end));
      let new_content: string =
        decoder.decode(content_as_bytes.slice(0, slice_start)) +
        `[[${target}|${slice_str}]]` +
        decoder.decode(content_as_bytes.slice(slice_end));

      let file_change: FileChange = {
        file_path: source,
        new_content: new_content,
        colored_content: colored_content,
      };
      if (perform_link) {
        if (accept_all[source] == undefined) {
          accept_all[source] = false;
        }
        if (decline_all[source] == undefined) {
          decline_all[source] = false;
        }
        if (accept_all[source]) {
          let tfile: TFile = tfilemap[source];
          file_content = new_content;

          await this.app.vault.modify(tfile, new_content);
          byte_increament += increment;
          continue;
        }
        if (decline_all[source]) {
          let json_link_serialized = link.serialize();
          remaining_links.push(json_link_serialized);
          continue;
        }


        let modal = new ParseModal(this, file_change);
        modal.open();
        
        await modal.wait_for_submit();

        if (modal.all_accepted) {
          // console.log("all accepted");
          accept_all[source] = true;
        }
        if (modal.all_declined) {
          // console.log("all declined");
          decline_all[source] = true;
        }

        if (modal.accepted) {
          let tfile: TFile = tfilemap[source];
          file_content = new_content;

          await this.app.vault.modify(tfile, new_content);
          byte_increament += increment;
        }

        if (modal.declined) {
          let json_link_serialized = link.serialize();
          remaining_links.push(json_link_serialized);
        }
      } else {
        let json_link_serialized = link.serialize();
        remaining_links.push(json_link_serialized);
      }
    }
    this.cache_obj[file_path] = {
      time: tfilemap[file_path].stat.mtime,
      links: remaining_links,
      aliases: alias_map[file_path],
    };
    await this.write_cache();
  }
  async write_cache() {
    if (!this.app.vault.adapter.exists(this.cache_path)) {
      await this.app.vault.adapter.write(this.cache_path, "{}");
    }
    await this.app.vault.adapter.write(
      this.cache_path,
      JSON.stringify(this.cache_obj),
    );
  }
  async read_cache() {
    if (!(await this.app.vault.adapter.exists(this.cache_path))) {
      await this.app.vault.adapter.write(this.cache_path, "{}");
    }
    await this.app.vault.adapter.read(this.cache_path);
  }
}

class ParseModal extends Modal {
  plugin: RustPlugin;
  change: FileChange;

  accepted: boolean;
  declined: boolean;

  all_accepted: boolean;
  all_declined: boolean;

  constructor(plugin: RustPlugin, file_change: FileChange) {
    super(plugin.app);
    this.plugin = plugin;
    this.change = file_change;
    this.accepted = false;
    this.declined = false;
    this.all_accepted = false;
    this.all_declined = false;
  }

  async onOpen() {
    const { contentEl } = this;
    new Setting(contentEl).addButton((btn) =>
      btn
        .setButtonText("Accept")
        .setCta()
        .onClick(() => {
          this.close();
          this.accepted = true;
        }),
    );

    new Setting(contentEl).addButton((btn) =>
      btn
        .setButtonText("Decline")
        .setCta()
        .onClick(() => {
          this.close();
          this.declined = true;
        }),
    );
    new Setting(contentEl).addButton((btn) =>
      btn
        .setButtonText("Accept All")
        .setCta()
        .onClick(() => {
          this.close();
          this.all_accepted = true;
          this.accepted = true;
        }),
    );

    new Setting(contentEl).addButton((btn) =>
      btn
        .setButtonText("Decline All")
        .setCta()
        .onClick(() => {
          this.close();
          this.all_declined = true;
          this.declined = true;
        }),
    );

    await MarkdownRenderer.renderMarkdown(
      this.change.colored_content,
      contentEl,
      "/",
      this.plugin,
    );
  }

  async wait_for_submit() {
    while (!this.accepted && !this.declined) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }

  onClose() {
    const { contentEl } = this;
    contentEl.empty();
  }
}

class ParseErrorModal extends Modal {
  plugin: RustPlugin;
  paths: string[];
  errors: string[];

  constructor(plugin: RustPlugin, paths: string[], errors: string[]) {
    super(plugin.app);
    this.plugin = plugin;
    this.paths = paths;
    this.errors = errors;
  }

  async onOpen() {
    const { contentEl } = this;
    for (let index = 0; index < this.paths.length; index++) {
      let path = this.paths[index];
      let error = this.errors[index];
      contentEl.createEl("h2", { text: path });
      contentEl.createEl("p", { text: error });
    }
  }

  onClose() {
    const { contentEl } = this;
    contentEl.empty();
  }
}

class RustPluginSettingTab extends PluginSettingTab {
  plugin: RustPlugin;

  constructor(app: App, plugin: RustPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;
    containerEl.empty();

    new Setting(containerEl)
      .setName("Case insensitive")
      .setDesc("Whether to use a case insensitive search when linking files")
      .addToggle((text) =>
        text
          .setValue(this.plugin.settings.caseInsensitive)
          .onChange(async (value) => {
            this.plugin.settings.caseInsensitive = value;
            await this.plugin.saveSettings();
          }),
      );
    new Setting(containerEl)
      .setName("Color of links")
      .setDesc(
        'Color to show links in the preview (no effect on the actual file), any supported CSS color is valid. Default is "red", but could also use hex: "#2ecc71"',
      )
      .addText((text) =>
        text.setValue(this.plugin.settings.color).onChange(async (value) => {
          this.plugin.settings.color = value;
          await this.plugin.saveSettings();
        }),
      );
    new Setting(containerEl)
      .setName("Include paths")
      .setDesc("Paths to include in linking, default is all files in the vault")
      .addTextArea((text) =>
        text
          .setValue(this.plugin.settings.includePaths)
          .onChange(async (value) => {
            this.plugin.settings.includePaths = value;
            await this.plugin.saveSettings();
          }),
      );
    new Setting(containerEl)
      .setName("Exclude paths")
      .setDesc("Paths to exclude in linking, default is no files in the vault")
      .addTextArea((text) =>
        text
          .setValue(this.plugin.settings.excludePaths)
          .onChange(async (value) => {
            this.plugin.settings.excludePaths = value;
            await this.plugin.saveSettings();
          }),
      );
  }
}

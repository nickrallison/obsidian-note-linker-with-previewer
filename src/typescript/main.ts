import { App, Component, Editor, MarkdownRenderer, MarkdownView, Modal, Notice, Plugin, PluginSettingTab, Setting, TFile, Vault } from 'obsidian';
import * as plugin from "../../pkg/obsidian_note_linker_with_previewer.js";

// DO NOT REMOVE, PLUGIN DOES NOT LOAD WITHOUT IT
import rustPlugin from "../../pkg/obsidian_note_linker_with_previewer_bg.wasm";

class RustPluginSettings {
	caseInsensitive: boolean;
	color: string;
	includePaths: string;

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

const DEFAULT_SETTINGS: RustPluginSettings = new RustPluginSettings(true, "red");

interface FileChange {
	file_path: string;
	new_content: string;
	colored_content: string;
}

export default class RustPlugin extends Plugin {
	settings: RustPluginSettings;

	async onload() {
		await this.loadSettings();

		// DO NOT REMOVE, PLUGIN DOES NOT LOAD WITHOUT IT
		const buffer = Buffer.from(rustPlugin, 'base64')
		// DO NOT REMOVE, PLUGIN DOES NOT LOAD WITHOUT IT
		await plugin.default(Promise.resolve(buffer));

		// Cleaning the cache
		let cache: string = this.app.vault.configDir + '/plugins/obsidian-note-linker-with-previewer/cache.json';
		let cache_string: string = await this.app.vault.adapter.read(cache);
		let cache_obj = JSON.parse(cache_string);
		for (let key in cache_obj) {
			if (!this.app.vault.getAbstractFileByPath(key)) {
				delete cache_obj[key];
			}
		}
		await this.app.vault.adapter.write(cache, JSON.stringify(cache_obj));

		this.addCommand({
			id: "link_current_file",
			name: "Link Current Note",
			callback: () => {
				let active_view = this.app.workspace.getActiveViewOfType(MarkdownView);
				if (active_view) {
					this.run_linker_on_file(active_view.file.path);
				}
			}
		});

		this.addCommand({
			id: "link_vault",
			name: "Link Vault",
			callback: () => {
				this.run_linker()
			}
		});

		this.addCommand({
			id: "scan_vault",
			name: "Scan Vault",
			callback: () => {
				this.scan_vault()
			}
		});

		this.addSettingTab(new RustPluginSettingTab(this.app, this));
	}

	async run_linker() {
		let cache: string = this.app.vault.configDir + '/plugins/obsidian-note-linker-with-previewer/cache.json';
		// if cache file does not exist, create it
		if (!await this.app.vault.adapter.exists(cache)) {
			await this.app.vault.adapter.write(cache, '{}');
		}

		// save file if it is open
		let active_view = this.app.workspace.getActiveViewOfType(MarkdownView);
		if (active_view) {
			await active_view.save();
		}


		let cache_string: string = await this.app.vault.adapter.read(cache);
		let cache_obj = JSON.parse(cache_string);
		let link_to_self = false;
		let filelist: TFile[] = this.app.vault.getMarkdownFiles();
		let filtered_filelist: TFile[] = [];
		let includePaths: string[];
		if (this.settings.includePaths == "") {
			includePaths = [];
		}
		else {
			includePaths = this.settings.includePaths.split("\n");
		}

		// if include paths is empty, include all files
		// otherwise only add files that are prefixed by one of the include paths
		if (includePaths.length > 0) {
			for (let file of filelist) {
				for (let path of includePaths) {
					if (file.path.startsWith(path)) {
						filtered_filelist.push(file);
						break;
					}
				}
			}
			filelist = filtered_filelist;
		}

		let tfilemap: { [key: string]: TFile } = {};
		for (let file of filelist) {
			tfilemap[file.path] = file;
		}
		let file_paths: string[] = filelist.map(file => file.path);

		let file_map: { [key: string]: string } = {};
		for (let file of filelist) {
			file_map[file.path] = await this.app.vault.cachedRead(file);
		}
		let wasm_vault: plugin.JsVault = plugin.JsVault.default();

		let total_files = filelist.length;
		let index = 1;
		for (let file of filelist) {
			wasm_vault.add_file(file.path, file_map[file.path]);
			index++;
		}

		let files: plugin.JsFile[] = [];
		for (let file_path of file_paths) {
			files.push(wasm_vault.get_file(file_path));
		}
		let settings_obj = new plugin.JsSettings(this.settings.caseInsensitive, link_to_self, this.settings.color);
		let valid_file_paths: string[] = wasm_vault.get_valid_file_paths();
		let valid_files: plugin.JsFile[] = valid_file_paths.map(path => wasm_vault.get_file(path));
		let link_finder: plugin.JsLinkFinder = new plugin.JsLinkFinder(valid_file_paths, valid_files, settings_obj);

		let byte_increament_map: { [key: string]: number } = {};

		let valid_files_len = valid_files.length;
		let valid_index = 1;
		for (let file_path of valid_file_paths) {
			// if file is in cache, is up to date, and has no links, skip
			// tfilemap[file_path].stat.mtime; <= cached last modified time
			if (cache_obj[file_path] && cache_obj[file_path]['links'].length == 0 && tfilemap[file_path].stat.mtime <= cache_obj[file_path]['time']) {
				console.log(`(${valid_index} / ${valid_files_len}) up to date ` + file_path);
				valid_index++;
				continue;
			}

			let file: plugin.JsFile = wasm_vault.get_file(file_path);
			let file_links: plugin.JsLink[];

			if (cache_obj[file_path]) {
				let last_modified = cache_obj[file_path]['time'];
				let file_last_modified = tfilemap[file_path].stat.mtime;
				// if last modified time is the same or earlier as cache
				if (file_last_modified <= last_modified) {
					let file_links_serialized: string[] = cache_obj[file_path]['links'];
					file_links = file_links_serialized.map(link_serialized => new plugin.JsLink(link_serialized));
				}
				else {
					file_links = link_finder.find_links(file);
				}
			}
			else {
				file_links = link_finder.find_links(file);
			}
			console.log(`(${valid_index} / ${valid_files_len}) Found Links for ` + file_path);
			new Notice(`(${valid_index} / ${valid_files_len}) Found Links for ` + file_path);
			valid_index++;
			let remaining_links: string[] = []
			for (let link of file_links) {
				let file_increment: number = 0;
				if (byte_increament_map[file_path]) {
					file_increment = byte_increament_map[file_path];
				}
				let slice_start = file_increment + link.get_start();
				let slice_end = file_increment + link.get_end();
				let source = link.get_source();
				let target = link.get_target();
				let content = file_map[source];

				let encoder = new TextEncoder();
				let decoder = new TextDecoder();
				let byteArray = encoder.encode(content);

				let content_as_bytes = encoder.encode(content);


				let slicedArray = byteArray.slice(slice_start, slice_end);
				let slice_str = decoder.decode(slicedArray);
				let replace_str = `[[${target}|${slice_str}]]`;

				let replaced_as_bytes = encoder.encode(replace_str);
				let increment = replaced_as_bytes.length - (slice_end - slice_start);
				let color = this.settings.color;
				let colored_content = decoder.decode(content_as_bytes.slice(0, slice_start)) + `<span style="color:${color}">\\[\\[${target}\\|${slice_str}\\]\\]</span>` + decoder.decode(content_as_bytes.slice(slice_end));
				let new_content: string = decoder.decode(content_as_bytes.slice(0, slice_start)) + `[[${target}|${slice_str}]]` + decoder.decode(content_as_bytes.slice(slice_end));

				let file_change: FileChange = {
					file_path: source,
					new_content: new_content,
					colored_content: colored_content
				}
				let modal = new ParseModal(this, file_change);
				modal.open();

				await modal.wait_for_submit();

				if (modal.accepted) {
					let tfile: TFile = this.app.vault.getAbstractFileByPath(source) as TFile;
					file_map[source] = new_content;

					await this.app.vault.process(tfile, () => new_content);
					if (byte_increament_map[source]) {
						byte_increament_map[source] += increment;
					} else {
						byte_increament_map[source] = increment;
					}
				}

				if (modal.declined) {
					let json_link_serialized = link.serialize();
					remaining_links.push(json_link_serialized);
				}
			}
			cache_obj[file_path] = {
				'time': tfilemap[file_path].stat.mtime,
				'links': remaining_links
			}
			await this.app.vault.adapter.write(cache, JSON.stringify(cache_obj));
		}
	}

	async run_linker_on_file(active_file_path: string) {
		let cache: string = this.app.vault.configDir + '/plugins/obsidian-note-linker-with-previewer/cache.json';
		// if cache file does not exist, create it
		if (!await this.app.vault.adapter.exists(cache)) {
			await this.app.vault.adapter.write(cache, '{}');
		}

		// save file if it is open
		let active_view = this.app.workspace.getActiveViewOfType(MarkdownView);
		if (!active_view) {
			return;
		}
		await active_view.save();

		let cache_string: string = await this.app.vault.adapter.read(cache);
		let cache_obj = JSON.parse(cache_string);
		let link_to_self = false;
		let filelist: TFile[] = this.app.vault.getMarkdownFiles();
		let filtered_filelist: TFile[] = [];
		let includePaths: string[] = [];

		let tfilemap: { [key: string]: TFile } = {};
		for (let file of filelist) {
			tfilemap[file.path] = file;
		}
		let file_paths: string[] = filelist.map(file => file.path);

		let file_map: { [key: string]: string } = {};
		for (let file of filelist) {
			file_map[file.path] = await this.app.vault.cachedRead(file);
		}
		let wasm_vault: plugin.JsVault = plugin.JsVault.default();

		let total_files = filelist.length;
		let index = 1;
		for (let file of filelist) {
			wasm_vault.add_file(file.path, file_map[file.path]);
			index++;
		}

		let files: plugin.JsFile[] = [];
		for (let file_path of file_paths) {
			files.push(wasm_vault.get_file(file_path));
		}
		let settings_obj = new plugin.JsSettings(this.settings.caseInsensitive, link_to_self, this.settings.color);
		let valid_file_paths: string[] = wasm_vault.get_valid_file_paths();
		let valid_files: plugin.JsFile[] = valid_file_paths.map(path => wasm_vault.get_file(path));
		let link_finder: plugin.JsLinkFinder = new plugin.JsLinkFinder(valid_file_paths, valid_files, settings_obj);

		let byte_increament_map: { [key: string]: number } = {};


		valid_file_paths = [active_file_path];

		let valid_files_len = valid_file_paths.length;
		let valid_index = 1;

		for (let file_path of valid_file_paths) {
			// if file is in cache, is up to date, and has no links, skip
			// tfilemap[file_path].stat.mtime; <= cached last modified time
			if (cache_obj[file_path] && cache_obj[file_path]['links'].length == 0 && tfilemap[file_path].stat.mtime <= cache_obj[file_path]['time']) {
				console.log(`(${valid_index} / ${valid_files_len}) up to date ` + file_path);
				valid_index++;
				continue;
			}

			let file: plugin.JsFile = wasm_vault.get_file(file_path);
			let file_links: plugin.JsLink[];

			if (cache_obj[file_path]) {
				let last_modified = cache_obj[file_path]['time'];
				let file_last_modified = tfilemap[file_path].stat.mtime;
				// if last modified time is the same or earlier as cache
				if (file_last_modified <= last_modified) {
					let file_links_serialized: string[] = cache_obj[file_path]['links'];
					file_links = file_links_serialized.map(link_serialized => new plugin.JsLink(link_serialized));
				}
				else {
					file_links = link_finder.find_links(file);
				}
			}
			else {
				file_links = link_finder.find_links(file);
			}
			console.log(`(${valid_index} / ${valid_files_len}) Found Links for ` + file_path);
			new Notice(`(${valid_index} / ${valid_files_len}) Found Links for ` + file_path);
			valid_index++;
			let remaining_links: string[] = []
			for (let link of file_links) {
				let file_increment: number = 0;
				if (byte_increament_map[file_path]) {
					file_increment = byte_increament_map[file_path];
				}
				let slice_start = file_increment + link.get_start();
				let slice_end = file_increment + link.get_end();
				let source = link.get_source();
				let target = link.get_target();
				let content = file_map[source];

				let encoder = new TextEncoder();
				let decoder = new TextDecoder();
				let byteArray = encoder.encode(content);

				let content_as_bytes = encoder.encode(content);


				let slicedArray = byteArray.slice(slice_start, slice_end);
				let slice_str = decoder.decode(slicedArray);
				let replace_str = `[[${target}|${slice_str}]]`;

				let replaced_as_bytes = encoder.encode(replace_str);
				let increment = replaced_as_bytes.length - (slice_end - slice_start);
				let color = this.settings.color;
				let colored_content = decoder.decode(content_as_bytes.slice(0, slice_start)) + `<span style="color:${color}">\\[\\[${target}\\|${slice_str}\\]\\]</span>` + decoder.decode(content_as_bytes.slice(slice_end));
				let new_content: string = decoder.decode(content_as_bytes.slice(0, slice_start)) + `[[${target}|${slice_str}]]` + decoder.decode(content_as_bytes.slice(slice_end));

				let file_change: FileChange = {
					file_path: source,
					new_content: new_content,
					colored_content: colored_content
				}
				let modal = new ParseModal(this, file_change);
				modal.open();

				await modal.wait_for_submit();

				if (modal.accepted) {
					let tfile: TFile = this.app.vault.getAbstractFileByPath(source) as TFile;
					file_map[source] = new_content;

					await this.app.vault.process(tfile, () => new_content);
					if (byte_increament_map[source]) {
						byte_increament_map[source] += increment;
					} else {
						byte_increament_map[source] = increment;
					}
				}

				if (modal.declined) {
					let json_link_serialized = link.serialize();
					remaining_links.push(json_link_serialized);
				}
			}
			cache_obj[file_path] = {
				'time': tfilemap[file_path].stat.mtime,
				'links': remaining_links
			}
			await this.app.vault.adapter.write(cache, JSON.stringify(cache_obj));
		}
	}

	async scan_vault() {
		let cache: string = this.app.vault.configDir + '/plugins/obsidian-note-linker-with-previewer/cache.json';
		// if cache file does not exist, create it
		if (!await this.app.vault.adapter.exists(cache)) {
			await this.app.vault.adapter.write(cache, '{}');
		}
		let link_to_self = false;
		let filelist: TFile[] = this.app.vault.getMarkdownFiles();
		let filtered_filelist: TFile[] = [];
		let includePaths: string[];
		if (this.settings.includePaths == "") {
			includePaths = [];
		}
		else {
			includePaths = this.settings.includePaths.split("\n");
		}

		// if include paths is empty, include all files
		// otherwise only add files that are prefixed by one of the include paths
		if (includePaths.length > 0) {
			for (let file of filelist) {
				for (let path of includePaths) {
					if (file.path.startsWith(path)) {
						filtered_filelist.push(file);
						break;
					}
				}
			}
			filelist = filtered_filelist;
		}

		let tfilemap: { [key: string]: TFile } = {};
		for (let file of filelist) {
			tfilemap[file.path] = file;
		}
		let file_paths: string[] = filelist.map(file => file.path);

		let file_map: { [key: string]: string } = {};
		for (let file of filelist) {
			file_map[file.path] = await this.app.vault.cachedRead(file);
		}
		let wasm_vault: plugin.JsVault = plugin.JsVault.default();

		let total_files = filelist.length;
		let index = 1;
		for (let file of filelist) {
			wasm_vault.add_file(file.path, file_map[file.path]);
			index++;
		}

		let files: plugin.JsFile[] = [];
		for (let file_path of file_paths) {
			files.push(wasm_vault.get_file(file_path));
		}
		let settings_obj = new plugin.JsSettings(this.settings.caseInsensitive, link_to_self, this.settings.color);
		let valid_file_paths: string[] = wasm_vault.get_valid_file_paths();
		let valid_files: plugin.JsFile[] = valid_file_paths.map(path => wasm_vault.get_file(path));
		let link_finder: plugin.JsLinkFinder = new plugin.JsLinkFinder(valid_file_paths, valid_files, settings_obj);

		let valid_files_len = valid_files.length;
		let valid_index = 1;
		for (let file_path of valid_file_paths) {
			let cache_string: string = await this.app.vault.adapter.read(cache);
			let cache_obj = JSON.parse(cache_string);
			let file: plugin.JsFile = wasm_vault.get_file(file_path);
			let file_links: plugin.JsLink[];

			// if file is in cache
			if (cache_obj[file_path]) {
				let last_modified = cache_obj[file_path]['time'];
				let file_last_modified = tfilemap[file_path].stat.mtime;
				// if last modified time is the same or earlier as cache
				if (file_last_modified <= last_modified) {
					let file_links_serialized: string[] = cache_obj[file_path]['links'];
					file_links = file_links_serialized.map(link_serialized => new plugin.JsLink(link_serialized));
				}
				else {
					file_links = link_finder.find_links(file);
				}
			}
			else {
				file_links = link_finder.find_links(file);
			}
			console.log(`(${valid_index} / ${valid_files_len}) Found Links for ` + file_path);
			new Notice(`(${valid_index} / ${valid_files_len}) Found Links for ` + file_path);
			valid_index++;
			let remaining_links: string[] = []
			for (let link of file_links) {
				let json_link_serialized = link.serialize();
				remaining_links.push(json_link_serialized);
			}
			cache_obj[file_path] = {
				'time': tfilemap[file_path].stat.mtime,
				'links': remaining_links
			}
			await this.app.vault.adapter.write(cache, JSON.stringify(cache_obj));
		}
	}

	async loadSettings() {
		this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}


}

class ParseModal extends Modal {
	plugin: RustPlugin;
	change: FileChange;

	accepted: boolean;
	declined: boolean;

	constructor(plugin: RustPlugin, file_change: FileChange) {
		super(plugin.app);
		this.plugin = plugin;
		this.change = file_change;
		this.accepted = false;
		this.declined = false;
	}

	async onOpen() {

		const { contentEl } = this;

		new Setting(contentEl)
			.addButton((btn) =>
				btn
					.setButtonText("Accept")
					.setCta()
					.onClick(() => {

						this.close();
						this.accepted = true;
					}));

		new Setting(contentEl)
			.addButton((btn) =>
				btn
					.setButtonText("Decline")
					.setCta()
					.onClick(() => {
						this.close();
						this.declined = true;
					}));

		await MarkdownRenderer.renderMarkdown(this.change.colored_content, contentEl, "/", this.plugin);;

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
			.setName('Case insensitive')
			.setDesc('Whether to use a case insensitive search when linking files')
			.addToggle(text => text
				.setValue(this.plugin.settings.caseInsensitive)
				.onChange(async (value) => {
					this.plugin.settings.caseInsensitive = value;
					await this.plugin.saveSettings();
				}));
		new Setting(containerEl)
			.setName('Color of links')
			.setDesc('Color to show links in the preview (no effect on the actual file), any supported CSS color is valid. Default is "red", but could also use hex: "#2ecc71"')
			.addText(text => text
				.setValue(this.plugin.settings.color)
				.onChange(async (value) => {
					this.plugin.settings.color = value;
					await this.plugin.saveSettings();
				}));
		new Setting(containerEl)
			.setName('Include Paths')
			.setDesc('Paths to include in linking, default is all files in the vault')
			.addTextArea(text => text
				.setValue(this.plugin.settings.includePaths)
				.onChange(async (value) => {

					this.plugin.settings.includePaths = value;
					await this.plugin.saveSettings();
				}));
	}
}

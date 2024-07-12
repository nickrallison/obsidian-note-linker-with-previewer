import { App, Component, Editor, MarkdownRenderer, MarkdownView, Modal, Notice, Plugin, PluginSettingTab, Setting, TFile, Vault } from 'obsidian';

import rustPlugin from "../../pkg/obsidian_note_linker_with_previewer_bg.wasm";
import * as plugin from "../../pkg/obsidian_note_linker_with_previewer.js";

// Remember to rename these classes and interfaces!

class RustPluginSettings {
	caseInsensitive: boolean;
	color: string;

	constructor(caseInsensitive: boolean, color: string) {
		this.caseInsensitive = caseInsensitive;
		this.color = color;
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
		// Instantiates the given module, which can either be bytes or a precompiled WebAssembly.Module.
		const buffer = Buffer.from(rustPlugin, 'base64')
		await plugin.default(Promise.resolve(buffer));
		plugin.onload(this);

		this.addCommand({
			id: "link_vault",
			name: "Link Vault",
			callback: () => {
				this.run_linker()
			}
		});

		this.addCommand({
			id: "list config",
			name: "List Config",
			callback: () => {
				this.list_config()
			}
		});

		this.addSettingTab(new RustPluginSettingTab(this.app, this));
	}

	async run_linker() {
		// json file with structure cache[path] = {...}
		// cache[path]['time'] = last_modified_time
		// cache[path]['links_remaining'] = remaining links to link
		let cache: string = this.app.vault.configDir + '/plugins/obsidian-note-linker-with-previewer/cache.json';
		// if cache file does not exist, create it
		if (!await this.app.vault.adapter.exists(cache)) {
			await this.app.vault.adapter.write(cache, '{}');
		}
		let cache_string: string = await this.app.vault.adapter.read(cache);
		let cache_obj = JSON.parse(cache_string);
		let link_to_self = false;
		console.log('Parsing Files 1');
		let settings = new plugin.JsSettings(this.settings.caseInsensitive, link_to_self, this.settings.color);
		console.log('Settings Created');
		let filelist: TFile[] = this.app.vault.getMarkdownFiles();
		console.log('Files Found');
		let tfilemap: { [key: string]: TFile } = {};
		for (let file of filelist) {
			tfilemap[file.path] = file;
		}
		let file_paths: string[] = filelist.map(file => file.path);
		console.log('Files Mapped');

		let file_map: { [key: string]: string } = {};
		for (let file of filelist) {
			file_map[file.path] = await this.app.vault.cachedRead(file);
			console.log('Read ' + file.path);
		}
		console.log('Files Read');
		let wasm_vault: plugin.JsVault = plugin.JsVault.default();
		console.log('Vault Created')

		let total_files = filelist.length;
		let index = 1;
		for (let file of filelist) {
			console.log(`(${index} / ${total_files}) Parsing ` + file.path);
			wasm_vault.add_file(file.path, file_map[file.path]);
			index++;
		}

		let files: plugin.JsFile[] = [];
		for (let file_path of file_paths) {
			files.push(wasm_vault.get_file(file_path));
		}
		console.log('Created File[] vec');
		let settings_obj = new plugin.JsSettings(this.settings.caseInsensitive, link_to_self, this.settings.color);
		console.log('Created Settings Object');
		let valid_file_paths: string[] = wasm_vault.get_valid_file_paths();
		let valid_files: plugin.JsFile[] = valid_file_paths.map(path => wasm_vault.get_file(path));
		console.log('Got Valid Files');
		let link_finder: plugin.JsLinkFinder = new plugin.JsLinkFinder(valid_file_paths, valid_files, settings_obj);
		console.log('Created Link Finder');

		let byte_increament_map: { [key: string]: number } = {};
		console.log('Getting Links, One Moment Please...');

		let valid_files_len = valid_files.length;
		let valid_index = 1;
		for (let file_path of valid_file_paths) {
			// if file is in cache and
			// last modified time is the same or earlier as cache
			// skip searching and get links from cache
			let file: plugin.JsFile = wasm_vault.get_file(file_path);
			let file_links: plugin.JsLink[];

			// if file is in cache
			if (cache_obj[file_path]) {
				let last_modified = cache_obj[file_path]['time'];
				let file_last_modified = tfilemap[file_path].stat.mtime;
				// if last modified time is the same or earlier as cache
				if (file_last_modified <= last_modified) {
					file_links = cache_obj[file_path]['links'];
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
			let remaining_links: plugin.JsLink[] = []
			for (let link of file_links) {
				let file_increment: number = 0;
				if (byte_increament_map[file_path]) {
					file_increment = byte_increament_map[file_path];
				}
				let slice_start = file_increment + link.get_start();
				let slice_end = file_increment + link.get_end();
				console.log('Link Found: ' + (file_increment + slice_start) + ' ' + (file_increment + slice_end));
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
				// <span style=color:#2ecc71>This is a test</span>  
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
					console.log('Accepted changes for ' + source);
					let tfile: TFile = this.app.vault.getAbstractFileByPath(source) as TFile;
					file_map[source] = new_content;
					await this.app.vault.modify(tfile, new_content);
					if (byte_increament_map[source]) {
						byte_increament_map[source] += increment;
					} else {
						byte_increament_map[source] = increment;
					}
				}

				if (modal.declined) {
					console.log('Declined changes for ' + source);
					remaining_links.push(link);
				}
			}
			cache_obj[file_path] = {
				'time': tfilemap[file_path].stat.mtime,
				'links': remaining_links
			}
			await this.app.vault.adapter.write(cache, JSON.stringify(cache_obj));
		}
	}

	async list_config() {
		let config_dir: string = this.app.vault.configDir + '/plugins/obsidian-note-linker-with-previewer';
		let config_files = await this.app.vault.adapter.list(config_dir);
		console.log(config_files);
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

	onSubmit() {
		console.log('Submitting');
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

		containerEl.createEl('h2', { text: 'Settings for my awesome plugin.' });


		new Setting(containerEl)
			.setName('Case Insensitive')
			.setDesc('Whether to use a case Insensitive search when linking files')
			.addToggle(text => text
				.setValue(this.plugin.settings.caseInsensitive)
				.onChange(async (value) => {
					this.plugin.settings.caseInsensitive = value;
					await this.plugin.saveSettings();
				}));
		new Setting(containerEl)
			.setName('Color of Links')
			.setDesc('Color to show links in the preview (no effect on the actual file), any supported CSS color is valid. Default is "red", but could also use hex: "#2ecc71".')
			.addText(text => text
				.setValue(this.plugin.settings.color)
				.onChange(async (value) => {
					this.plugin.settings.color = value;
					await this.plugin.saveSettings();
				}));
	}
}

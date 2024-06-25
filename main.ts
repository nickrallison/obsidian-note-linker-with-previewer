import { App, Editor, MarkdownView, Modal, Notice, Plugin, PluginSettingTab, Setting, TFile } from 'obsidian';

import rustPlugin from "./pkg/obsidian_linker_plugin_bg.wasm";
import * as plugin from "./pkg/obsidian_linker_plugin.js";

// Remember to rename these classes and interfaces!

interface RustPluginSettings {
	mySetting: string;
}

const DEFAULT_SETTINGS: RustPluginSettings = {
	mySetting: 'default'
}

export default class RustPlugin extends Plugin {
	settings: RustPluginSettings;

	async onload() {
		await this.loadSettings();

		// init wasm
		// const buffer = Uint8Array.from(atob(rustPlugin), c => c.charCodeAt(0))
		// await plugin.default(Promise.resolve(buffer));
		// plugin.onload(this);

		const buffer = Buffer.from(rustPlugin, 'base64')
		await plugin.default(Promise.resolve(buffer));
		plugin.onload(this);

		this.addCommand({
			id: "open-note-linker",
			name: "Open",
			callback: () => {
				new ParseModal(this.app).open();
			}
		});
	}

	async loadSettings() {
		this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}


}

class ParseModal extends Modal {
	constructor(app: App) {
		super(app);
	}

	onOpen() {
		const { contentEl } = this;
		let res = plugin.parse_files_to_str(this.app.vault);
		let example_struct = new plugin.ExampleStruct("test");
		res = res + ", " + example_struct.do_thing();
		contentEl.setText(res);
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
			.setName('Setting #1')
			.setDesc('It\'s a secret')
			.addText(text => text
				.setPlaceholder('Enter your secret')
				.setValue(this.plugin.settings.mySetting)
				.onChange(async (value) => {
					console.log('Secret: ' + value);
					this.plugin.settings.mySetting = value;
					await this.plugin.saveSettings();
				}));
	}
}
